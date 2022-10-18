export class Widget extends Element {
  value = 0;
  constructor(props) {
    super(props);
    this.props = props;
    this.value = props.value;
    console.log("Constructor ", props, this);
  }

  sendValue(postfix) {
    if (postfix) postfix = "|" + postfix;
    else postfix = "";
    Window.this.xcall("update_uniform", this.props.id + postfix, this.value.toString());
  }
}