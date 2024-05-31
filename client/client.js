const port = 8080;
const style_tag_id = "sage/css-css";
// Don't change the above. Dynamically written on Runtime from yeti.json

// Setups up socket for live reload of CSS from websocket server file watcher
// Create a WebSocket connection to the websocket server
const socket = new WebSocket(`ws://localhost:${port}/ws`);
console.log("Socket script loaded");

socket.onopen = function (event) {
  console.log("Connection established");
};

// Listen for messages from the Rust websocket server
socket.onmessage = function (event) {
  const message = event.data;
  console.info(`Received message from server: ${message}`);

  switch (message) {
    case "reload":
      console.info("Reloading css");
      const styleElement = document.getElementById(style_tag_id);

      // Exit if style element not found
      if (!styleElement) {
        console.error(
          `Reload failed. Failed to find element with id: ${style_tag_id}`
        );
        return;
      }

      const url = styleElement.getAttribute("href");

      // Convert timestamp from milliseconds to seconds to mimic PHP time()
      const timestampAsSeconds = Math.floor(new Date().getTime() / 1000);

      // Add URL query to cache bust
      const url_query = `${url}?ver=${timestampAsSeconds}`;

      fetch(url_query)
        .then((res) => res.text())
        .then((css) => {
          // styleElement.setAttribute("href", url);
          styleElement.textContent = css;
          console.info("CSS reloaded");
        })
        .catch((e) => console.error("Error reloading css. ", e));
      break;
    default:
      break;
  }
};

socket.onerror = function (error) {
  console.error("WebSocket error: ", error);
};

socket.onclose = function (event) {
  console.log("Connection closed", event.data);
};