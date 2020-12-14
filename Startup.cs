using auto_highlighter_back_end.Services;
using Azure.Messaging.ServiceBus;
using Azure.Storage.Blobs;
using Microsoft.AspNetCore.Builder;
using Microsoft.AspNetCore.Hosting;
using Microsoft.AspNetCore.Http;
using Microsoft.Extensions.Configuration;
using Microsoft.Extensions.DependencyInjection;
using Microsoft.Extensions.Hosting;
using Microsoft.Extensions.Logging;
using System;
using System.Collections.Generic;
using System.IO;
using System.Linq;
using System.Threading.Tasks;
using Xabe.FFmpeg;
using Xabe.FFmpeg.Downloader;

namespace HighlightProcessService
{
    public class Startup
    {
        private readonly IConfiguration _config;
        private readonly IWebHostEnvironment _env;
        public Startup(IConfiguration config, IWebHostEnvironment env)
        {
            _config = config;
            _env = env;
        }

        // This method gets called by the runtime. Use this method to add services to the container.
        // For more information on how to configure your application, visit https://go.microsoft.com/fwlink/?LinkID=398940
        public void ConfigureServices(IServiceCollection services)
        {


            services.AddLogging(loggingBuilder =>
            {
                loggingBuilder.AddConsole();
                loggingBuilder.AddDebug();
                loggingBuilder.AddAzureWebAppDiagnostics();
            });

            services.AddSingleton<IVideoProcessService, VideoProcessService>();

            services.AddSingleton(x => new ServiceBusClient(_config["ConnectionStrings:AzureServiceBus"]));
            services.AddSingleton<IMessageQueueService, MessageQueueService>();

            services.AddSingleton(x => new BlobServiceClient(_config["ConnectionStrings:AzureBlobStorage"]));
            services.AddSingleton<IBlobService, BlobService>();
        }

        // This method gets called by the runtime. Use this method to configure the HTTP request pipeline.
        public async void Configure(IApplicationBuilder app, IWebHostEnvironment env)
        {
            if (env.IsDevelopment())
            {
                app.UseDeveloperExceptionPage();
            }


            string ffmpegLocation = Path.Combine(_env.ContentRootPath, _config["FFmpegSettings:ExecutableLocation"]);

            if (!Directory.Exists(ffmpegLocation))
            {
                Directory.CreateDirectory(ffmpegLocation);
                await FFmpegDownloader.GetLatestVersion(FFmpegVersion.Official, ffmpegLocation);
            }

            FFmpeg.SetExecutablesPath(ffmpegLocation, ffmpegExeutableName: "FFmpeg");
        }
    }
}
