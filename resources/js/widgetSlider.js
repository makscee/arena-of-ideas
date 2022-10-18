import { Widget } from "./widget";

export class WidgetSlider extends Widget {
  constructor(props) {
    super(props);
  }

  render() {
    console.log("Render", this, this.props);
    return <widgetSlider class="widget">
      <div><h3>{this.props.name || "[No name]"}</h3><h4>{this.props.id}</h4></div>
      <div>
        <input type="hslider" min={this.props.from} max={this.props.to} step={this.props.step} value={this.value} />
        <p>{+this.value.toFixed(2)}</p>
      </div>
    </widgetSlider>;
  }

  ["on change at input"](event, input) {
    this.componentUpdate({ value: input.value });
    this.sendValue();
  }
}