export class Slider extends Element {
  val = 0;

  componentDidMount() {
    this.componentUpdate();
  }

  render() {
    return <div>
      <button id="t">t</button>
      <input type="hslider" step="0.01" />
      <h1>value: {this.val}</h1>
    </div>;
  }

  ["on change at input"](event, input) {
    console.log("input");
    this.componentUpdate({ val: input.value });
  }

  ["on click at button#t"]() {
    console.log("btn#t");
  }
}