import { WidgetEnum } from "./widgetEnum";
import { WidgetSlider } from "./widgetSlider";
import { WidgetVector } from "./widgetVector";

export class Panel extends Element {
    widgets = [

    ];

    addWidget(type) {
        this.widgets.push(type);
        this.componentUpdate();
    }

    removeWidget() {
        this.widgets.pop();
        this.componentUpdate();
    }

    componentDidMount() {
        document.on("click", "#save", function (event, input) {
            Window.this.xcall("save_uniforms");
        });
        this.componentUpdate();
    }

    render() {
        this.content(<></>);
        this.widgets.forEach(widget => {
            var element;
            console.log("render widget", widget.type, widget.value);
            switch (widget.type) {
                case "Vector":
                    element = <WidgetVector name={widget.name} id={widget.id} from={widget.from} to={widget.to} step={widget.step} value={widget.value} />;
                    break;
                case "Int":
                case "Float":
                    element = <WidgetSlider name={widget.name} id={widget.id} from={widget.from} to={widget.to} step={widget.step} value={widget.value} />;
                    break;
                case "Enum":
                    element = <WidgetEnum name={widget.name} id={widget.id} values={widget.values} value={widget.value} />;
                    break;
            }
            this.append(element);
        })
        this.append(<button id="save">Save</button>)
        // for (var i = 0; i < this.widgets.length; i++) {
        //     this.append(<widget>test {i}</widget>);
        // }
    }

    getValue() {
        return 33;
    }

    createWidget(args) {
        console.log("Add widget ", args.type);
        this.widgets.push(args);
        this.componentUpdate();
    }
}