# Tessera Architecture

The `tessera` project uses a multi-crate workspace structure with clear separation of responsibilities:

*   **`tessera`**: This is the core crate of the framework. It contains all the fundamental functionality required to build and run a `tessera` application. Its main responsibilities include:
    *   **Component Tree Management**: Defines the core data structure of the UI, `ComponentNodeTree`, to represent and manage the component hierarchy.
    *   **Rendering**: Contains the rendering logic to draw the component tree to the screen.
    *   **Runtime**: Provides the `TesseraRuntime`, which is responsible for driving the entire application lifecycle, including event handling, state updates, and redrawing.
    *   **Basic Types**: Defines units like `Dp`, `Px`, and modules for handling cursor, keyboard, and scroll states.

*   **`tessera_basic_components`**: This crate provides a set of ready-to-use basic UI components. Its main responsibility is to provide developers with the fundamental elements for building interfaces, similar to standard widget libraries in other UI frameworks. These components include:
    *   Layout components: `Row` and `Column` for horizontal and vertical arrangement of child components.
    *   Content components: `Text` for displaying static text, and `TextEditor` for text input.
    *   Functional components: `Spacer` for creating empty space, and `Surface`, which likely defines a drawable surface area.
    Developers build complex user interfaces by combining these basic components.

*   **`tessera_macros`**: This crate provides a key procedural macro, `#[tessera]`. Its responsibility is to simplify the component creation process and hide underlying complexity. When a developer marks a function with the `#[tessera]` macro, the macro automatically injects code at compile time to seamlessly integrate the function into the `tessera` component system. This includes automatically adding the component node to the global component tree and removing it after the function's scope ends, thus enabling declarative component definition.