window.onload = function (e) {
  let connect_button = document.getElementById("connect");
  let username_field = document.getElementById("username");
  let connection_status = document.getElementById("status");

  let board = document.getElementById("board");
  let chips = generate_chips(board);

  let drop_buttons = document.querySelectorAll(".drop-button");

  let socket = null;

  function drop_chip(column) {
    if (socket == null) {
      console.error("cannot drop chip when socket is closed");
      return;
    }
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
    if (msg.board != null) {
      display_chips(chips, msg.board);
    }
  }

  function buttons_connect(connected) {
    connect_button.disabled = connected;
  }

  function connect(username) {
    console.log(`Connecting as ${username}...`);
    let protocol = window.location.protocol == "https:" ? "wss" : "ws";
    socket = new WebSocket(
      `${protocol}://${window.location.host}/play/${username}`,
    );

    socket.onopen = function (e) {
      connection_status.classList.add("status-online");
      buttons_connect(true);
      console.log("Connected!");
    };

    socket.onmessage = function (e) {
      let msg = JSON.parse(e.data);
      handle_message(msg);
    };

    socket.onclose = function (e) {
      connection_status.classList.remove("status-online");
      console.log("Disconnected");
      buttons_connect(false);
    };

    socket.onerror = function (e) {
      console.error(e);
    };
  }

  connect_button.addEventListener("click", function (e) {
    let username = username_field.value;
    connect(username);
  });

  for (const [i, chip] of chips) {
    chip.addEventListener("click", function (e) {
      let col = e.target.getAttribute("col");
      drop_chip(parseInt(col));
    });
  }
};
