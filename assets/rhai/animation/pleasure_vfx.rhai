animator.duration(2.0);
animator.spawn_painter(`
    painter.color(0x00, 0xE6, 0x76, 0xFF);
    painter.alpha(1.0 - t);
    painter.scale_rect(1.3 - t);
    painter.circle(1.0);
    painter.paint();
`);
