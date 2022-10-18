import { Widget } from "./widget";

export class WidgetVector extends Widget {
    constructor(props) {
        super(props);
        this.valueX = props.value;
        this.valueY = props.value;
    }

    valueX = 0;
    valueY = 0;

    render() {
        console.log("Render", this, this.props);
        return <widgetVector class="widget">
            <div><h3>{this.props.name || "[No name]"}</h3><h4>{this.props.id}</h4></div>
            <div>
                <p>X</p>
                <input #x type="hslider" min={this.props.from} max={this.props.to} step={this.props.step} value={this.valueX} />
                <p>{+this.valueX.toFixed(2)}</p>
            </div>
            <div>
                <p>Y</p>
                <input #y type="hslider" min={this.props.from} max={this.props.to} step={this.props.step} value={this.valueY} />
                <p>{+this.valueY.toFixed(2)}</p>
            </div>
        </widgetVector>;
    }

    ["on change at input#x"](event, input) {
        this.componentUpdate({ valueX: input.value });
        this.sendValue("x");
    }

    ["on change at input#y"](event, input) {
        this.componentUpdate({ valueY: input.value });
        this.sendValue("y");
    }
}