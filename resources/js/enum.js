export class Enum extends Element {
    values = [

    ];
    currentValue = 0;

    componentDidMount() {
        this.values.push("one");
        this.values.push("two");
        this.values.push("three");
        this.componentUpdate();
    }

    render() {
        return <div>
            <button id="prev">&lt;</button>
            {this.values[this.currentValue]}
            <button id="next">&gt;</button>
        </div>;
    }

    ["on click at button#next"](event, input) {
        this.componentUpdate({ currentValue: (this.currentValue + 1) % this.values.length });
    }

    ["on click at button#prev"](event, input) {
        this.componentUpdate({ currentValue: (this.currentValue - 1 + this.values.length) % this.values.length });
    }
}