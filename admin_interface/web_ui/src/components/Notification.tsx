import {useEffect, useState} from "react";
import {Alert, Snackbar, Stack} from "@mui/material";

type Message = {
  severity: 'info' | 'error' | 'warning' | 'success';
  message: string;
  timeout: number;
  key: string;
}

const eventBus = new MessageChannel();

export function notify(severity: Message['severity'], message: string, timeout?: number, key?: string) {
  eventBus.port2.postMessage({severity, message, timeout, key: key || performance.now().toString()});
}

export default function Notification() {
  const [messages, setMessages] = useState([] as Message[]);

  useEffect(() => {
    const messageHandler = (event: MessageEvent<Message>) => {
      setMessages(messages => {
        // Replace existing message
        if (event.data.key) {
          const message = messages.find(message => message.key === event.data.key);
          if (message) {
            message.severity = event.data.severity;
            message.message = event.data.message;
            message.timeout = event.data.timeout;
            return messages.slice(0);
          }
        }
        return [...messages, event.data];
      });

      // Remove message after timeout
      if (event.data.timeout !== undefined) {
        setTimeout(() => {
          setMessages((messages) => messages.filter(message => message.key !== event.data.key));
        }, event.data.timeout)
      }
    };

    eventBus.port1.addEventListener('message', messageHandler);
    eventBus.port1.start();

    return () => {
      eventBus.port1.removeEventListener('message', messageHandler);
      eventBus.port1.close();
    }
  }, []);

  const handleClose = (messageKey: string) => {
    setMessages(messages => messages.filter(message => message.key !== messageKey))
  }

  return (
    <Stack
      spacing={2}
      sx={{position: 'fixed', bottom: '32px', left: '50%', transform: 'translateX(-50%)', zIndex: 1301}}
    >
      {messages.map(message =>
        <Alert
          key={message.key}
          severity={message.severity}
          onClose={() => handleClose(message.key)}
        >{message.message}</Alert>
      )}
    </Stack>
  )
}