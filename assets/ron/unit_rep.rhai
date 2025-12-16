// Unit representation painter script
// Draws a unit with HP, Power, and Stacks indicators

// White outer circle with feathering
painter.color(255, 255, 255, 255);
painter.hollow(2.0);
painter.feathering(20.0);
painter.circle(1.0);

// Inner circle with less feathering
painter.hollow(1.0);
painter.feathering(0.0);
painter.circle(1.0);
painter.paint();

// Dark background circles for HP
painter.hollow(0.0);
painter.color(25, 25, 25, 255);
painter.circle(0.24);

// Red HP indicator
painter.hollow(2.0);
painter.color(213, 0, 0, 255);
painter.circle(0.24);

// HP text
painter.color(255, 255, 255, 255);
painter.text("[b 10]");
painter.translate(0.68, 0.85);
painter.paint();

// Dark background circle for Power
painter.hollow(0.0);
painter.color(25, 25, 25, 255);
painter.circle(0.24);

// Orange Power indicator
painter.hollow(2.0);
painter.color(255, 145, 0, 255);
painter.circle(0.24);

// Power text
painter.color(255, 255, 255, 255);
painter.text("[b 5]");
painter.translate(-0.68, 0.85);
painter.paint();
