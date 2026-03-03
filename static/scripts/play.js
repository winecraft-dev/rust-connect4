window.onload = function (e) {
  let connect_button = document.getElementById("connect");
  let username_field = document.getElementById("username");
  let connection_status = document.getElementById("status");

  let gameplay_text = document.getElementById("gameplay");

  let socket = null;

  function drop_chip(column) {
    socket.send(
      JSON.stringify({
        type: "DropChip",
        column: column,
      }),
    );
  }

  function handle_message(msg) {
    console.log(`Received message:`);
    console.log(msg);
    if (typeof msg.board !== undefined) {
      gameplay_text.value = msg.board;
    }
  }

  function connect(username) {
    console.log(`Connecting as ${username}...`);
    socket = new WebSocket(`ws://${window.location.host}/play/${username}`);

    socket.onopen = function (e) {
      connection_status.classList.add("status-online");
      console.log("Connected!");
    };

    socket.onmessage = function (e) {
      let msg = JSON.parse(e.data);
      handle_message(msg);
    };

    socket.onclose = function (e) {
      connection_status.classList.remove("status-online");
      console.log("Disconnected");
    };

    socket.onerror = function (e) {
      console.error(e);
    };
  }

  connect_button.addEventListener("click", function (e) {
    let username = username_field.value;
    connect(username);
  });
};
