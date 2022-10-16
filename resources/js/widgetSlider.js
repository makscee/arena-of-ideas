import { Widget } from "./widget";

export class WidgetSlider extends Widget {
  constructor(props) {
    super(props);
  }

  render() {
    console.log("Render", this, this.props);
    return <widgetSlider>
      <input type="hslider" min={this.props.from} max={this.props.to} step={this.props.step} value={this.value} />
      <h1>value: {this.value}</h1>
    </widgetSlider>;
  }

  ["on change at input"](event, input) {
    console.log("input");
    this.componentUpdate({ value: input.value });
    this.sendValue();
  }
}