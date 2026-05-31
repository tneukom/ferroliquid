Port of
LiquidSketch (free
on [iOS Appstore](https://apps.apple.com/us/app/liquidsketch/id544717096),
[Android Appstore](https://play.google.com/store/apps/details?id=net.tobiasneukom.liquidsketch))
fluid simulation to Rust with an egui interface for experimentation.

**[Run in browser](https://tneukom.github.io/ferroliquid)**

https://github.com/user-attachments/assets/9ce14bdb-bd3e-4bff-818a-93f1196d4752

Features

- Flexible boundary conditions using a signed distance field
- FLIP (fluid implicit particle) method
- [Conjugate gradient](https://en.wikipedia.org/wiki/Conjugate_gradient_method)
  (with preconditioner) pressure solver
- Liquid surface reconstruction using OpenGL
- Color advection using OpenGL
- Widgets
    - Gravity force
    - Arbitrary radial force with function editor
    - Uniform force
    - Inflow


