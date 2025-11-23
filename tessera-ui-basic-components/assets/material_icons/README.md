# [@material-design-icons/svg](https://github.com/marella/material-design-icons/tree/main/svg)

Latest optimized SVGs for material design icons.

> This package is automatically updated, so it will always have the latest icons from Google.

> For Material Symbols, see [`material-symbols`](https://www.npmjs.com/package/material-symbols)
>
> For icon fonts, see [`@material-design-icons/font`](https://www.npmjs.com/package/@material-design-icons/font)

- [Installation](#installation)
- [Usage](#usage)
- [Available Icons](#available-icons)

## Installation

Install the [latest version][releases] using:

```sh
npm install @material-design-icons/svg@latest
```

## Usage

Material icons are available in five styles&mdash;`filled`, `outlined`, `round`, `sharp` and `two-tone`. SVGs for each icon are located in the corresponding style directory:

```js
@material-design-icons/svg/{style}/{icon}.svg
```

For example, SVGs for `face` icon are located at:

```js
@material-design-icons/svg/filled/face.svg
@material-design-icons/svg/outlined/face.svg
@material-design-icons/svg/round/face.svg
@material-design-icons/svg/sharp/face.svg
@material-design-icons/svg/two-tone/face.svg
```

### React

You can use [`@svgr/webpack`](https://www.npmjs.com/package/@svgr/webpack) to import SVGs as React components:

```js
import Face from '@material-design-icons/svg/filled/face.svg';

// Face is a React component that renders the SVG

function App() {
  return (
    <div className="App">
      <Face />
    </div>
  );
}
```

If you are using `@svgr/webpack` with `file-loader` (example: [Create React App](https://create-react-app.dev/docs/adding-images-fonts-and-files/#adding-svgs)), use:

```js
import { ReactComponent as Face } from '@material-design-icons/svg/filled/face.svg';
```

### Vue

You can use [`@svgv/webpack`](https://www.npmjs.com/package/@svgv/webpack) to import SVGs as Vue components:

```html
<template>
  <div class="App">
    <Face />
  </div>
</template>

<script>
import Face from '@material-design-icons/svg/filled/face.svg';

// Face is a Vue component that renders the SVG

export default {
  components: {
    Face,
  },
};
</script>
```

### Styling

SVGs can be styled using CSS. Setting `width` and `height` to `1em` allows changing size via `font-size`. Setting `fill` to `currentColor` allows changing color via `color`.

```css
.App svg {
  width: 1em;
  height: 1em;
  fill: currentColor;
}
```

## Available Icons

See [demo].

## License

Material design icons are created by [Google](https://github.com/google/material-design-icons#license).

> We have made these icons available for you to incorporate into your products under the [Apache License Version 2.0][license]. Feel free to remix and re-share these icons and documentation in your products.
We'd love attribution in your app's *about* screen, but it's not required.

[releases]: https://github.com/marella/material-design-icons/releases
[license]: https://github.com/marella/material-design-icons/blob/main/svg/LICENSE
[demo]: https://marella.github.io/material-design-icons/demo/svg/
