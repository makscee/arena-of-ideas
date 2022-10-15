export class Widget extends Element {
  constructor(props, kids) {
    super(props, kids);
  }

  componentDidMount() {
    this.content(this.innerHTML);
  }
}