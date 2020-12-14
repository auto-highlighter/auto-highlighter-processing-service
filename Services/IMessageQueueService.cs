using System.Threading.Tasks;

namespace auto_highlighter_back_end.Services
{
    public interface IMessageQueueService
    {
        Task ReceiveMessagesAsync();
        Task SendMessageAsync(byte[] messageBody);
    }
}