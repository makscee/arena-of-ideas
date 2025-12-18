animator.spawn_painter(`
    painter.scale_rect(2.0);
    painter.color("#a70101ab");
    painter.alpha(4.0 - t * 4.0);
    for i in 0..30 {
        painter.circle(0.2);
        painter.scale_mesh(t);
        let v = unit_vec(i.random(0.0, pi() * 2.0)) * min(t * (i + 1).random(1.0, 2.0), 0.9);
        painter.translate(v);
        painter.paint();
    }
`);
