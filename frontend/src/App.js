import './App.css';
import React, { useState, useEffect } from "react";
import ReconnectingWebSocket from "reconnecting-websocket";

const Chat = () => {
  const [socket, setSocket] = useState(null);
  const [messages, setMessages] = useState([]);
  //const [sentMessages, setSentMessages] = useState([]); // [message, message, ...
  const [input, setInput] = useState('');

  useEffect(() => {
    const ws = new ReconnectingWebSocket("ws://localhost:8080");
    setSocket(ws);

    ws.onmessage = (event) => {
      setMessages((prevMessages) => [...prevMessages, event.data]);
    };

    return () => {
      ws.close();
    };
  }, []);

  const sendMessage = (e) => {
    e.preventDefault();
    if (input.trim() !== '') {
      socket.send(input);
      setMessages((prevMessages) => [...prevMessages, input]);
      setInput('');
    }
  };

  return (
    <div>
      <h2>Chat</h2>
      <div>
        {messages.map((message, index) => (
          <div key={index}>{message}</div>
        ))}
      </div>
      <form onSubmit={sendMessage}>
        <input
          type="text"
          value={input}
          onChange={(e) => setInput(e.target.value)}
          placeholder="Type your message..."
        />
        <button type="submit">Send</button>
      </form>
    </div>
  );
};

function App() {
  return (
    <div className="App">
      <Chat />
    </div>
  );
}

export default App;