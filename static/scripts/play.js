window.onload = function (e) {
  let connect_button = document.getElementById("connect");
  let username_field = document.getElementById("username");

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

  let status = init_status();
  let chips = init_chips(drop_chip);

  function handle_message(msg) {
    console.log(`Received message:`);
    console.log(msg);
    if (msg.board != null) {
      chips.display(msg.board);
    }
    if (msg.type == "MatchMade") {
      status.matchmade(msg);
    }
    if (msg.type == "Board") {
      status.turn(msg.turn);
    }
    if (msg.type == "Moved") {
      status.turn(opposite_color(msg.last_mover));
    }
    if (msg.type == "Won") {
      status.win(msg.winner);
    }
    if (msg.type == "Stalemate") {
      status.win(null);
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
      buttons_connect(true);
      chips.clear();
      status.reset(username);
      console.log("Connected!");
    };

    socket.onmessage = function (e) {
      let msg = JSON.parse(e.data);
      handle_message(msg);
    };

    socket.onclose = function (e) {
      console.log("Disconnected");
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

  document
    .getElementById("username-form")
    .addEventListener("submit", function (e) {
      e.preventDefault();
    });
};
