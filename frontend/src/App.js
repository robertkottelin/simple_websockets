import logo from './logo.svg';
import './App.css';
import React, { useState, useEffect } from "react";
import ReconnectingWebSocket from "reconnecting-websocket";

const Chat = () => {
  const [socket, setSocket] = useState(null);

  useEffect(() => {
    const ws = new ReconnectingWebSocket("ws://localhost:8080");
    setSocket(ws);
    return () => {
      ws.close();
    };
  }, []);

  // Rest of the component logic goes here

  return (
    <div>
      {/* Chat UI elements go here */}
    </div>
  );
};


function App() {
  return (
    <div className="App">
      <header className="App-header">
        <img src={logo} className="App-logo" alt="logo" />
        <p>
          Edit <code>src/App.js</code> and save to reload.
        </p>
        <a
          className="App-link"
          href="https://reactjs.org"
          target="_blank"
          rel="noopener noreferrer"
        >
          Learn React
        </a>
      </header>
      <Chat />
    </div>
    
  );
}

export default App;
