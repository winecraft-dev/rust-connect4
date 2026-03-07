window.onload = function (e) {
  let connect_button = document.getElementById("connect");
  let username_field = document.getElementById("username");
  let status = document.getElementById("status");

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
    if (msg.turn != null) {
      status_turn(msg.turn);
    } else if (msg.last_mover != null) {
      if (msg.last_mover == "Red") {
        status_turn("Blue");
      } else {
        status_turn("Red");
      }
    } else if (msg.winner != null) {
      status_win(msg.winner);
    } else if (msg.type == "Stalemate") {
      status_win(null);
    }
  }

  function buttons_connect(connected) {
    connect_button.disabled = connected;
  }

  function status_disconnected() {
    status.style.backgroundColor = "yellow";
    status.innerHTML = "Disconnected";
  }

  function status_waiting() {
    status.style.backgroundColor = "green";
    status.innerHTML = "Waiting...";
  }

  function status_turn(color) {
    if (color == "Red") status.style.backgroundColor = "red";
    else if (color == "Blue") status.style.backgroundColor = "blue";
    status.innerHTML = "Player Turn";
  }

  function status_win(color) {
    if (color == null) {
      status.style.backgroundColor = "gray";
      status.innerHTML = "Stalemate";
      return;
    }
    if (color == "Red") status.style.backgroundColor = "red";
    else if (color == "Blue") status.style.backgroundColor = "blue";
    status.innerHTML = "Winner";
  }

  function connect(username) {
    console.log(`Connecting as ${username}...`);
    let protocol = window.location.protocol == "https:" ? "wss" : "ws";
    socket = new WebSocket(
      `${protocol}://${window.location.host}/play/${username}`,
    );

    socket.onopen = function (e) {
      status_waiting();
      buttons_connect(true);
      console.log("Connected!");
    };

    socket.onmessage = function (e) {
      let msg = JSON.parse(e.data);
      handle_message(msg);
    };

    socket.onclose = function (e) {
      console.log("Disconnected");
      status_disconnected();
      buttons_connect(false);
      socket = null;
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

  status_disconnected();
};
