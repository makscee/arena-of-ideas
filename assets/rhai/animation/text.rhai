animator.duration(3.5);
animator.spawn_painter(`
    painter.color(0xFF, 0xFF, 0xFF, 0xFF);
    painter.alpha(4.0 - t * 4.0);
    painter.text("sample text");
    painter.scale_mesh(1.0);
    painter.translate([0.0, -0.7 - t * 1.1]);
    painter.paint();
`);
