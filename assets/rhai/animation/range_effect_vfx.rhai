animator.spawn_painter(`
    let t = t_unit;
    painter.color(255, 255, 255, 255);
    painter.feathering(50.0 * t);
    painter.alpha(1.0 - t);
    painter.curve(0.1, 2.0);
    painter.paint();
`);
animator.spawn_painter(`
    let t = t_unit;
    painter.color(255, 255, 255, 255);
    painter.alpha(if t < 0.1 { 1.0 } else { 0.0 });
    painter.curve(0.1, 2.0);
    painter.paint();
`);
