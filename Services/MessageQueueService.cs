using System;
using System.Collections.Generic;
using System.Linq;
using System.Threading.Tasks;
using Azure.Messaging.ServiceBus;
using HighlightProcessService.DTOs;
using Microsoft.Extensions.Configuration;
using Microsoft.Extensions.Logging;
using Utf8Json;

namespace auto_highlighter_back_end.Services
{
    public class MessageQueueService : IMessageQueueService
    {
        private readonly ILogger _logger;
        private readonly ServiceBusClient _serviceBusClient;
        private readonly IConfiguration _config;
        private readonly IVideoProcessService _videoProcessService;

        public MessageQueueService(ILogger<IMessageQueueService> logger, ServiceBusClient serviceBusClient, IConfiguration config, IVideoProcessService videoProcessService)
        {
            _logger = logger;
            _serviceBusClient = serviceBusClient;
            _config = config;
            _videoProcessService = videoProcessService;
        }
        public async Task SendMessageAsync(byte[] messageBody)
        {
            _logger.LogInformation("Started sending message");

            ServiceBusSender sender = _serviceBusClient.CreateSender(_config["ServiceBus:QueueName"]);

            ServiceBusMessage message = new ServiceBusMessage(messageBody);

            try
            {
                await sender.SendMessageAsync(message);
                _logger.LogInformation("Message sent");
            }
            catch (Exception e)
            {
                _logger.LogInformation($"Failed to send {e.Message}");
            }
        }


        public async Task ReceiveMessagesAsync()
        {
            ServiceBusProcessor processor = _serviceBusClient.CreateProcessor(_config["ServiceBus:QueueName"], new ServiceBusProcessorOptions());

            processor.ProcessMessageAsync += MessageHandler;
            processor.ProcessErrorAsync += ErrorHandler;

            await processor.StartProcessingAsync();
        }

        // handle received messages
        private async Task MessageHandler(ProcessMessageEventArgs args)
        {
            string body = args.Message.Body.ToString();


            _logger.LogInformation($"Recieved message {body}");
            try
            {

                ProccessVodDTO proccessVodDTO = JsonSerializer.Deserialize<ProccessVodDTO>(body);

                if (proccessVodDTO is not null)
                {
                    await _videoProcessService.ProcessHightlightAsync(proccessVodDTO);
                }
            }
            catch (Exception e)
            {
                _logger.LogInformation($"caught exception in message {body} processing: {e.Message}");
            }

            await args.CompleteMessageAsync(args.Message);
        }

        // handle any errors when receiving messages
        private Task ErrorHandler(ProcessErrorEventArgs args)
        {
            Console.WriteLine(args.Exception.ToString());
            return Task.CompletedTask;
        }
    }
}