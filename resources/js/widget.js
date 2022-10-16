export class Widget extends Element {
  value = 0;
  constructor(props) {
    super(props);
    this.props = props;
    this.value = props.value;
    console.log("Constructor ", props, this);
  }

  sendValue() {
    Window.this.xcall("update_uniform", this.props.id, this.value.toString());
  }
}