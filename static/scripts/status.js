const RED = "Red";
const BLUE = "Blue";

const c_default = "#ccc";
const c_light_red = "#FDD";
const c_light_blue = "#DDF";
const c_red = "#F33";
const c_blue = "#55F";

function init_status() {
  let your_color = null;
  let opp_color = null;

  let your_status = document.getElementById("your-status");
  let opp_status = document.getElementById("opp-status");

  let your_name = document.getElementById("your-name");
  let opp_name = document.getElementById("opp-name");

  let status = document.getElementById("status");

  this.reset = function (username) {
    your_color = null;
    opp_color = null;
    your_name.innerHTML = username != null ? username : "???";
    opp_name.innerHTML = "???";
    status.innerHTML = username != null ? "Matchmaking" : "Disconnected";
    your_status.style.backgroundColor = c_default;
    opp_status.style.backgroundColor = c_default;
  };

  this.turn = function (color) {
    your_status.style.backgroundColor = convert_color(
      your_color,
      color == your_color,
    );
    opp_status.style.backgroundColor = convert_color(
      opp_color,
      color == opp_color,
    );
    status.innerHTML = your_color == color ? "Your Turn" : "Opp Turn";
  };

  this.matchmade = function (msg) {
    your_color = msg.your_color;
    opp_color = opposite_color(your_color);
    your_name.innerHTML = msg.your_username;
    opp_name.innerHTML = msg.opponent_username;
    status.innerHTML = "Match Made!";

    your_status.style.backgroundColor = convert_color(your_color, false);
    opp_status.style.backgroundColor = convert_color(opp_color, false);
  };

  this.win = function (winner) {
    if (winner == null) {
      your_status.style.backgroundColor = convert_color(your_color, false);
      opp_status.style.backgroundColor = convert_color(opp_color, false);
      status.innerHTML = "Stalemate!";
    } else {
      your_status.style.backgroundColor = convert_color(
        your_color,
        winner == your_color,
      );
      opp_status.style.backgroundColor = convert_color(
        opp_color,
        winner == opp_color,
      );
      status.innerHTML = winner == your_color ? "You Won!" : "You Lost!";
    }
  };

  this.reset(null);

  return this;
}

function convert_color(color, active) {
  if (color == "Red") {
    return active ? c_red : c_light_red;
  } else {
    return active ? c_blue : c_light_blue;
  }
}

function opposite_color(color) {
  if (color == "Red") {
    return "Blue";
  } else return "Red";
}
