using HighlightProcessService.DTOs;
using System;
using System.Threading.Tasks;

namespace auto_highlighter_back_end.Services
{
    public interface IVideoProcessService
    {
        Task ProcessHightlightAsync(ProccessVodDTO highlight);
    }
}