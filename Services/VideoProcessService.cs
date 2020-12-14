using Azure.Storage.Blobs.Models;
using HighlightProcessService.DTOs;
using Microsoft.AspNetCore.Hosting;
using Microsoft.Extensions.Configuration;
using Microsoft.Extensions.Logging;
using System;
using System.Collections.Generic;
using System.IO;
using System.Threading;
using System.Threading.Tasks;
using Utf8Json;
using Xabe.FFmpeg;

namespace auto_highlighter_back_end.Services
{
    public class VideoProcessService : IVideoProcessService
    {
        private readonly ILogger _logger;
        private readonly IConfiguration _config;
        private readonly IWebHostEnvironment _env;
        private readonly IBlobService _blobService;

        public VideoProcessService(ILogger<IVideoProcessService> logger, IConfiguration config, IWebHostEnvironment env, IBlobService blobService)
        {
            _logger = logger;
            _config = config;
            _env = env;
            _blobService = blobService;
        }
        public async Task ProcessHightlightAsync(ProccessVodDTO highlight)
        {
            _logger.LogInformation($"Processing video {highlight.Hid}");

            List<int> timestamps = await GetTimestamps(highlight.Hid);
            List<HighlightTimeSpan> highlightTimeSpans = ToHighlightTimeSpans(timestamps);

            await DownloadVodFromBlob(highlight.Hid);

            await EditVideo(highlight.Hid, highlightTimeSpans);

            await UploadEditedVod(highlight.Hid);

            _logger.LogInformation($"Finished video {highlight.Hid}");
        }

        private async Task EditVideo(Guid hid, List<HighlightTimeSpan> highlightTimeSpans)
        {
            string vodFilePath = Path.Combine(_env.ContentRootPath, _config.GetValue<string>("FileUploadLocation"), hid.ToString());

            IConversion conversion;

            int start;
            int duration;
            string[] fileNames = new string[highlightTimeSpans.Count];

            Task[] tasks = new Task[highlightTimeSpans.Count];
            for (int index = 0; index < highlightTimeSpans.Count; index++)
            {
                fileNames[index] = vodFilePath + index.ToString() + ".mp4";
                start = highlightTimeSpans[index].Start;
                duration = highlightTimeSpans[index].Duration;

                _logger.LogInformation($"Start Time: {start} | End Time: {start + duration} | File Name: {fileNames[index]}");

                conversion = await FFmpeg.Conversions.FromSnippet.Split(vodFilePath + ".mp4", fileNames[index], TimeSpan.FromMilliseconds(start), TimeSpan.FromMilliseconds(duration));

                tasks[index] = conversion.Start();
            }

            await Task.WhenAll(tasks);

            if (highlightTimeSpans.Count > 1)
            {
                File.Delete(vodFilePath + ".mp4");
                conversion = await FFmpeg.Conversions.FromSnippet.Concatenate(vodFilePath + ".mp4", fileNames);

                await conversion.Start();
            }

            foreach (string fileName in fileNames)
            {
                File.Delete(fileName);
            }


        }

        private List<HighlightTimeSpan> ToHighlightTimeSpans(List<int> timestamps)
        {
            List<HighlightTimeSpan> highlightTimeSpans = new();
            int highlightLength = int.Parse(_config["HighlightSettings:HighlightLength"]);
            int startTime = -1;
            for (int index = 0; index < timestamps.Count; index++)
            {
                if (startTime == -1)
                {
                    startTime = timestamps[index] - highlightLength;

                    if (startTime < 0)
                    {
                        startTime = 0;
                    }
                }

                if (index == timestamps.Count - 1 || timestamps[index + 1] - highlightLength > timestamps[index])
                {
                    highlightTimeSpans.Add(new(startTime, timestamps[index] - startTime));
                    startTime = -1;
                }
            }
            return highlightTimeSpans;
        }

        private async Task<List<int>> GetTimestamps(Guid hid)
        {
            BlobDownloadInfo blob = await _blobService.GetBlobAsync(_config["BlobSettings:ContainerName"], hid.ToString() + ".json");

            List<int> timestamps = JsonSerializer.Deserialize<List<int>>(blob.Content);
            return timestamps;
        }

        private async Task DownloadVodFromBlob(Guid hid)
        {

            string vodFilePath = Path.Combine(_env.ContentRootPath, _config.GetValue<string>("FileUploadLocation"), hid.ToString() + ".mp4");

            BlobDownloadInfo vod = await _blobService.GetBlobAsync(_config["BlobSettings:ContainerName"], hid.ToString() + ".mp4");

            using Stream vodFileStream = new FileStream(vodFilePath, FileMode.Create);

            await vod.Content.CopyToAsync(vodFileStream);
        }

        private async Task UploadEditedVod(Guid hid)
        {

            string vodFilePath = Path.Combine(_env.ContentRootPath, _config.GetValue<string>("FileUploadLocation"), hid.ToString() + ".mp4");

            using (Stream vod = File.OpenRead(vodFilePath))
            {

                await _blobService.UploadFileBlobAsync(
                        _config["BlobSettings:ContainerName"],
                        vod,
                        "video/mp4",
                        hid + ".mp4");
            }

            File.Delete(vodFilePath);
        }

    }

    public struct HighlightTimeSpan
    {
        public HighlightTimeSpan(int start, int duration)
        {
            Start = start;
            Duration = duration;
        }

        public int Start { get; }
        public int Duration { get; }
    }
}
