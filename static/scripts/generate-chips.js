function init_chips(drop_chip) {
  let board = document.getElementById("board");
  let chips = new Map();
  for (let r = 5; r >= 0; r--) {
    for (let c = 0; c < 7; c++) {
      let chip = document.createElement("div");
      let id = chip_id(r, c);
      chip.id = id;
      chip.className = "chip";
      chip.setAttribute("col", c);

      chips.set(id, chip);
      board.appendChild(chip);
    }
  }

  this.display = function (board_layout) {
    for (let r = 0; r < 6; r++) {
      for (let c = 0; c < 7; c++) {
        let id = chip_id(r, c);
        let color = board_layout[c][r];
        if (color == null) continue;
        else if (color == "Red") chips.get(id).classList.add("chip-red");
        else if (color == "Blue") chips.get(id).classList.add("chip-blue");
      }
    }
  };

  this.clear = function () {
    for (const [i, chip] of chips) {
      chip.classList.remove("chip-red");
      chip.classList.remove("chip-blue");
    }
  };

  for (const [i, chip] of chips) {
    chip.addEventListener("click", function (e) {
      let col = e.target.getAttribute("col");
      drop_chip(parseInt(col));
    });
  }

  return this;
}

function chip_id(row, col) {
  return `${row},${col}`;
}
