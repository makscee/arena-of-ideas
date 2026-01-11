animator.duration(3.0);
animator.spawn_painter(`
    let t = t_unit;
    painter.color(0xFF, 0xFF, 0xFF, 0xFF);
    painter.alpha(2.0 - t * 2.0);
    painter.curve(0.1, -1.5);
    painter.paint();
`);
animator.spawn_painter(`
    let t = t_unit;
    painter.alpha(3.0 - t * 3.0);
    painter.hollow(1.0);
    painter.circle(1.4);
    painter.scale_mesh(1.0 - 0.4 * t);
    painter.translate([0.0, 0.0]);
    painter.paint();
`);
