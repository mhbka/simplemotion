# SimpleMotion Notes

## Project layout

- `simplemotion-lib` is the native Rust library. It contains the parser and
  resolved resource graph; raw JSON types are private implementation details.
- `simplemotion-cli` is a temporary native tree inspector. It is useful for
  exploring documents without building the WASM project.
- The root package is still the WASM wrapper and re-exports the native library.

Run the native tests with:

```sh
cargo test --workspace
```

Inspect the sample with:

```sh
cargo run -p simplemotion-cli -- examples/sd002.json
```

## Resolved model

`parse_motion_resources` is the public entry point. It parses JSON into a
private raw representation, then resolves resource strings into stable slot-map
keys.

`MotionResources` currently assumes that the document has exactly one object.
Its resolved contents are placed at the root level:

- `object_type` is the object's exporter type.
- `roots` contains graph-root `MotionKey`s.
- `motion_key(label)` finds a motion by exported name.
- `motion(key)` retrieves a resolved motion.
- `icon_key(path)` and `icon(key)` retrieve image resources.

The raw `object` map is retained only because it matches the JSON format.
Documents with zero or multiple objects are rejected during resolution.

## Resource graph

There are two slot-mapped resource kinds:

- `MotionKey`: a playable motion with a layer tree.
- `IconKey`: an image resource.

A frame source resolves to one of:

- `ResourceReference::Icon(IconKey)` for `src/<source-group>/<icon-name>`.
- `ResourceReference::Motion(MotionKey)` for `motion/<object>/<motion-name>`.
- `ResourceReference::Layout` for the special `layout` value.
- `ResourceReference::Unresolved(String)` for values not yet understood.

The sample's raw icon layout is:

```text
source["SD002"].icon["bg"]
```

This resolves to the frame path `src/SD002/bg`. Icon groups are raw-only; the
resolved model flattens the actual icons directly into `IconKey`s.

## Motion hierarchy

Playback begins from a selected root motion, then recursively evaluates its
layers and any nested motions:

```text
root motion
  -> layer tree
    -> timed keyframes
      -> icon, nested motion, or layout
```

The current `roots` list means motions with no incoming `MotionKey` reference.
For `sd002.json`, this produces five graph roots:

```text
SD002AA, SD002AB, SD002BA, SD002BB, swing2suma
```

`swing2suma` is unreferenced, so the graph alone cannot prove which roots are
user-selectable animations. Do not hard-code the four `SD002??` names unless a
reliable exporter rule or metadata source is found.

## Layer types observed in the sample

These names are based on consistent observed frame contents:

| Exported value | Rust variant | Observed role |
| --- | --- | --- |
| `0` | `LayerType::Icon` | Layer draws icon resources. |
| `2` | `LayerType::Layout` | Layer contains the special `layout` source. |
| `3` | `LayerType::Motion` | Layer contains nested motion references. |
| `12` | `LayerType::Mask` | `mask` layer using icon frames. |

## Frame types observed in the sample

| Exported value | Rust variant | Observed role |
| --- | --- | --- |
| `0` | `FrameType::Empty` | No content; likely clears/hides the layer's current resource. |
| `2` | `FrameType::ParentTransformsStatically` | Sets the frame resource's parent-side state until another frame changes it. |
| `3` | `FrameType::ParentTransformsDynamically` | Animates a nested motion's parent-side state over time. |

The names describe the layer containing the reference, not whether the child
motion itself animates:

- A static-transform frame can mount a child motion that fully animates inside
  its own layer tree. Its descendants can also use dynamic-transform frames.
- A dynamic-transform frame additionally animates the whole mounted child
  motion's placement, angle, curves, or timing from the parent timeline.

In the sample, type-2 nested-motion frames are time-zero attachments with
basic fields and `timeOffset: 0`. Type-3 nested-motion frames occur in the
long `swing*` motions at times `0`, `45`, and `90`; they include fields such
as `coord`, `acc`, `ccc`, `angle`, and changing `timeOffset` values.

## Animation work still to do

Start with a simple recursive evaluator:

1. Choose one root `MotionKey`.
2. Traverse its layer tree recursively instead of collecting layers into a
   temporary vector.
3. At the current time, choose each layer's active keyframe.
4. Draw icon references and recursively evaluate motion references.
5. Initially skip `Layout` references and treat `Empty` as drawing nothing.

Important open questions:

- Exact child-motion local-time calculation. A likely starting hypothesis is
  `child_time = parent_time - frame_start_time + time_offset`, but this is not
  verified against the original runtime.
- Looping and clamping semantics for `lastTime` and `loopTime`.
- Exact interpolation behavior for dynamic-transform frames and fields such as
  `acc` and `ccc`.
- What `Layout` does beyond acting as a special non-image source.
- Mask rendering semantics for `LayerType::Mask` and frame `mask` values.
- A reliable definition of user-selectable roots versus merely unreferenced
  motions.
