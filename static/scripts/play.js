window.onload = function (e) {
  let connect_button = document.getElementById("connect");
  let username_field = document.getElementById("username");
  let connection_status = document.getElementById("status");

  let drop_buttons = document.querySelectorAll(".drop-button");
  console.log(drop_buttons);

  let gameplay_text = document.getElementById("gameplay");

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
      gameplay_text.value = msg.board;
    }
  }

  function buttons_connect(connected) {
    connect_button.disabled = connected;
    drop_buttons.forEach(function (drop_button) {
      drop_button.disabled = !connected;
    });
  }

  function connect(username) {
    console.log(`Connecting as ${username}...`);
    socket = new WebSocket(`ws://${window.location.host}/play/${username}`);

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

  drop_buttons.forEach(function (drop_button) {
    drop_button.addEventListener("click", function (e) {
      drop_chip(parseInt(e.target.getAttribute("column")));
    });
  });
};
