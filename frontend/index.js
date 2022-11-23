import * as wasm from "pdp12_web";
import { memory } from "pdp12_web/pdp12_web_bg.wasm";

async function svgDecal(id) {
  const embedding_element = document.getElementById(id);
  return new Promise((resolve) => {
    if (embedding_element.getSVGDocument() !== null) {
      resolve(embedding_element.getSVGDocument());
    }
    embedding_element.addEventListener("load", () => {
      resolve(embedding_element.getSVGDocument());
    });
  });
}

function getSwBits(side) {
  let num = 0;
  for (let i = 0; i < 12; i++) {
    if (i > 0) {
      num = num << 1;
    }
    if (document.getElementById(`${side}_${i}`).checked) {
      num |= 1;
    }
  }
  return num;
}

function getLsBits() {
  return getSwBits("ls");
}

function getRsBits() {
  return getSwBits("rs");
}
function saveMem(machine) {
  let memPtr = machine.dump_memory();
  let memBuffer = new Uint16Array(memory.buffer, memPtr, 4096);
  localStorage.setItem("memory", JSON.stringify(Array.from(memBuffer)));
}
function loadMem() {
  let jsonValue = localStorage.getItem("memory");
  try {
    let value = JSON.parse(jsonValue);
    if ((value != null && value.length != 4096) || value == null) {
      console.log(
        "Array not found or not right length, starting with fresh memory"
      );
      return new Uint16Array(4096);
    }
    return new Uint16Array(value);
  } catch (e) {
    console.log("Json parse error: ", e);
    return new Uint16Array(4096);
  }
}

async function main() {
  function examine(step = false) {
    let lsBits = getLsBits();
    machine.examine(lsBits, step);
  }
  function fill(step = false) {
    let lsBits = getLsBits();
    let rsBits = getRsBits();
    machine.fill(lsBits, rsBits, step);
    saveMem(machine);
  }
  function key_do() {
    let lsBits = getLsBits();
    machine.key_do(lsBits);
    saveMem(machine);
  }
  const decal = await svgDecal("decal");

  function setLights(id, number) {
    for (let i = 11; i >= 0; i--) {
      const light = decal.getElementById(`${id}_${i}`);
      if (number & 0b1) {
        light.setAttribute("fill", "url(#lightOn)");
      } else {
        light.setAttribute("fill", "url(#lightOff)");
      }
      number = number >> 1;
    }
  }
  window.uiFunctions = {};
  window.uiFunctions.setLightBits = function (which, bits) {
    setLights(which, bits);
  };
  window.uiFunctions.setLightBit = function (which, val) {
    if (val) {
      decal.getElementById(which).setAttribute("fill", "url(#lightOn)");
    } else {
      decal.getElementById(which).setAttribute("fill", "url(#lightOff)");
    }
  };
  window.uiFunctions.getNow = function (dummy) {
    return window.performance.now();
  };
  const machine = new wasm.Machine(loadMem());

  const clickAudios = [
    new Audio("resources/click0.mp3"),
    new Audio("resources/click1.mp3"),
    new Audio("resources/click2.mp3"),
    new Audio("resources/click3.mp3"),
    new Audio("resources/click4.mp3"),
    new Audio("resources/click5.mp3"),
  ];
  document.querySelectorAll('input[type="checkbox"').forEach((elem) =>
    elem.addEventListener("change", function (event) {
      clickAudios[Math.floor(Math.random() * 6)].play();
    })
  );
  document.querySelectorAll(".momentary").forEach((elem) => {
    elem.addEventListener("click", function (event) {
      event.preventDefault();
    });
    elem.addEventListener("mousedown", function (event) {
      this.checked = true;
      clickAudios[Math.floor(Math.random() * 6)].play();
      if (this.id === "exam") {
        examine();
      }
      if (this.id === "stepExam") {
        examine(true);
      }
      if (this.id === "fill") {
        fill();
      }
      if (this.id === "fillStep") {
        fill(true);
      }
      if (this.id === "do") {
        key_do();
      }
      if (this.id === "start_ls") {
        // Start machine
      }
    });
    elem.addEventListener("mouseup", function (event) {
      this.checked = false;
      clickAudios[Math.floor(Math.random() * 6)].play();
    });
  });

  document.addEventListener("keydown", function (event) {
    if (event.repeat) {
      return;
    }
    const lskeys = ["1", "2", "3", "4", "5", "6", "7", "8", "9", "0", "-", "="];
    const lskeyIndex = lskeys.indexOf(event.key);
    if (lskeyIndex >= 0) {
      document.getElementById(`ls_${lskeyIndex}`).click();
    }
    const rskeys = ["q", "w", "e", "r", "t", "y", "u", "i", "o", "p", "[", "]"];
    const rskeyIndex = rskeys.indexOf(event.key);
    if (rskeyIndex >= 0) {
      document.getElementById(`rs_${rskeyIndex}`).click();
    }
    if (event.key === "f") {
      document.getElementById("fillStep").checked = true;
      clickAudios[Math.floor(Math.random() * 6)].play();
      fill(true);
    }
    if (event.key === "g") {
      document.getElementById("fill").checked = true;
      clickAudios[Math.floor(Math.random() * 6)].play();
      fill();
    }
    if (event.key === "h") {
      document.getElementById("stepExam").checked = true;
      clickAudios[Math.floor(Math.random() * 6)].play();
      examine(true);
    }
    if (event.key === "j") {
      document.getElementById("exam").checked = true;
      clickAudios[Math.floor(Math.random() * 6)].play();
      examine();
    }
  });
  document.addEventListener("keyup", function (event) {
    if (event.key === "f") {
      document.getElementById("fillStep").checked = false;
      clickAudios[Math.floor(Math.random() * 6)].play();
    }
    if (event.key === "g") {
      document.getElementById("fill").checked = false;
      clickAudios[Math.floor(Math.random() * 6)].play();
    }
    if (event.key === "h") {
      document.getElementById("stepExam").checked = false;
      clickAudios[Math.floor(Math.random() * 6)].play();
    }
    if (event.key === "j") {
      document.getElementById("exam").checked = false;
      clickAudios[Math.floor(Math.random() * 6)].play();
    }
  });
}

main();
