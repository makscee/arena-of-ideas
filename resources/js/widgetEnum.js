import { Widget } from "./widget";

export class WidgetEnum extends Widget {
    values = [];
    ind = 0;

    constructor(props) {
        super(props);
    }

    render() {
        console.log("render enum", this.props);
        return <widgetEnum>
            <button id="prev">&lt;</button>
            {this.value}
            <button id="next">&gt;</button>
        </widgetEnum>;
    }

    ["on click at button#next"](event, input) {
        let ind = (this.ind + 1) % this.props.values.length;
        this.componentUpdate({
            ind: ind,
            value: this.props.values[ind]
        });
        this.sendValue();
    }

    ["on click at button#prev"](event, input) {
        let ind = (this.ind - 1 + this.props.values.length) % this.props.values.length;
        this.componentUpdate({
            ind: ind,
            value: this.props.values[ind]
        });
        this.sendValue();
    }
}