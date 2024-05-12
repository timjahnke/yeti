const port = 8080;

// Setups up socket for live reload of CSS from websocket server file watcher
// Create a WebSocket connection to the websocket server
const socket = new WebSocket(`ws://localhost:${port}`);
console.log("Socket script loaded");

socket.onopen = function (event) {
  console.log("Connection established");
};

// Listen for messages from the Rust websocket server
socket.onmessage = function (event) {
  const message = event.data;
  console.log("Received message from server: ", message);

  switch (message) {
    case "reload":
      console.log("Reloading css");
      // Convert timestamp from milliseconds to seconds to mimic PHP time()
      const timestampAsSeconds = Math.floor(new Date().getTime() / 1000);
      const url = `https://myapp.local/app/themes/sage-8/dist/main.css?ver=${timestampAsSeconds}`;

      fetch(url)
        .then((res) => res.text())
        .then((css) => {
          const styleElement = document.getElementById("sage/css-css");
          if (styleElement) {
            // styleElement.setAttribute("href", url);
            styleElement.textContent = css;
          } else {
            const newStyleElement = document.createElement("link");
            newStyleElement.setAttribute("id", "sage/css-css");
            styleElement.setAttribute("href", url);
            // newStyleElement.textContent = css;
            document.head.appendChild(newStyleElement);
          }
        })
        .catch((e) => console.error("Error reloading css. ", e));
      break;
  }
};

socket.onerror = function (error) {
  console.error("WebSocket error: ", error);
};

socket.onclose = function (event) {
  console.log("Connection closed", event.data);
};
