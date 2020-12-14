using Azure.Storage.Blobs.Models;
using System;
using System.IO;
using System.Threading.Tasks;

namespace auto_highlighter_back_end.Services
{
    public interface IBlobService
    {
        Task DeleteBlobAsync(string blobContainerName, string fileName);
        Task<BlobDownloadInfo> GetBlobAsync(string blobContainerName, string fileName);
        Task<Uri> UploadFileBlobAsync(string blobContainerName, Stream content, string contentType, string fileName);
    }
}