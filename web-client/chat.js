document.addEventListener("DOMContentLoaded",
  function()
  { 
    const socket = new WebSocket("ws://127.0.0.1:8080");
    socket.onmessage = function (event) {
    // #messages is the <textarea/>
    const messages = document.getElementById("messages"); // Append the received message
    // to the existing list of messages
      messages.value += event.data;
      messages.value += '\n';
    };
    const sendButton= document.getElementById("send"); sendButton.addEventListener("click", (event) => {
    // #message is the <input/>
    const message = document.getElementById("message"); socket.send(message.value)
    message.value = ""; // Clear the input box
    })
});
