let socket = null;

function drop_chip(column) {
  socket.send(
    JSON.stringify({
      type: "DropChip",
      column: column,
    }),
  );
}

function connect(username) {
  console.log(`Connecting as ${username}`);

  socket = new WebSocket("ws://localhost:8080/play/" + username);

  socket.onopen = function (e) {
    console.log("Connected");
  };

  socket.onmessage = function (e) {
    console.log(e.data);

    let msg = JSON.parse(e.data);

    if (typeof msg.board !== "undefined") {
      console.log(msg.board);
    }
  };

  socket.onclose = function (e) {
    console.log("Disconnected!");
  };

  socket.onerror = function (e) {
    console.error(e);
  };
}
