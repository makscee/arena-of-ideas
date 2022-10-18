import { Widget } from "./widget";

export class WidgetEnum extends Widget {
    values = [];
    ind = 0;

    constructor(props) {
        super(props);
    }

    render() {
        return <widgetEnum class="widget">
            <div><h3>{this.props.name || "[No name]"}</h3><h4>{this.props.id}</h4></div>
            <button id="prev">&lt;</button>
            <p>{this.value}</p>
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