# swc-plugin-mark-expressions
An SWC plugin to mark function calls with block comments

## Usage
See [example](https://github.com/dm33tri/swc-plugin-mark-expressions/tree/master/example) for usage with `webpack` and `swc-loader`.

```javascript
fn('arg');
```

gets transformed to:

```javascript
fn(/* mark: arg */ 'arg');
```

## @swc/core version compatibility

| @swc/core       | swc-plugin-mark-expressions |
|-----------------|-----------------------------|
| 1.3.68~1.3.80   | 0.1.0                       |
| 1.3.106~1.3.107 | 0.1.1                       |