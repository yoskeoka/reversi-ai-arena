import Phaser from "phaser";
import type { ReplayFrame } from "../replay/model";

export function mountBoard(element: HTMLElement, initial: ReplayFrame): (frame: ReplayFrame) => void {
  let frame = initial;
  class BoardScene extends Phaser.Scene {
    constructor() { super("board"); }
    create() { this.scale.on("resize", draw); draw(); }
  }
  const draw = () => {
    const graphics = scene?.add.graphics();
    if (!graphics) return;
    const size = Math.min(480, element.clientWidth || 480), cell = size / 8;
    graphics.fillStyle(0x1f6b45).fillRect(0, 0, size, size).lineStyle(1, 0x0d3823);
    for (let i = 0; i <= 8; i++) graphics.lineBetween(i * cell, 0, i * cell, size).lineBetween(0, i * cell, size, i * cell);
    frame.board.forEach((row, r) => row.forEach((disc, c) => { if (disc !== "empty") graphics.fillStyle(disc === "black" ? 0x152018 : 0xf2f0df).fillCircle((c + .5) * cell, (r + .5) * cell, cell * .38); }));
  };
  let scene: BoardScene | undefined;
  const game = new Phaser.Game({ type: Phaser.CANVAS, parent: element, width: 480, height: 480, transparent: true, scene: class extends BoardScene { create() { scene = this; super.create(); } } });
  return (next) => { frame = next; scene?.children.removeAll(); draw(); };
}
