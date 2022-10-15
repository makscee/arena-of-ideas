import { Widget } from "./widget";
import { Slider } from "./slider";
import { Enum } from "./enum";

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
        console.log("2 + 3 ", Window.this.xcall("calc_sum", 5, 10));
        document.on("click", "#minus", function (event, input) {
            var panel = document.$("panel");
            // @ts-ignore
            panel.removeWidget();
        });
        document.on("click", "#plus", function (event, input) {
            var panel = document.$("panel");
            var type = document.$("#w-type").value;
            console.log(type);
            // @ts-ignore
            panel.addWidget(type);
        });
        this.componentUpdate();
    }

    render() {
        this.content(<></>);
        this.widgets.forEach(type => {
            var element;
            switch (type) {
                case "text":
                    element = <widget>some text</widget>;
                    break;
                case "slider":
                    element = <widget><slider /></widget>;
                    break;
                case "enum":
                    element = <widget><enum /></widget>;
                    break;
            }
            this.append(element);
        })
        // for (var i = 0; i < this.widgets.length; i++) {
        //     this.append(<widget>test {i}</widget>);
        // }
    }

    getValue() {
        return 33;
    }
}