# PSB / e-mote Data Structure Description

> **Document index:** [`README.md`](README.md)  
> **Pose implementation:** [`MOTIONPLAYER_TEXTURE_WORLD_COORDS.md`](MOTIONPLAYER_TEXTURE_WORLD_COORDS.md)  
> **Architecture overview:** [`MOTIONPLAYER_ARCHITECTURE.md`](MOTIONPLAYER_ARCHITECTURE.md) §5

> File: `tests/test_files/emote/e-mote3.0バニラパジャマa.json`  
> Based on:
> - The JSON sample itself
> - Current parsing implementation: `cpp/plugins/motionplayer/EmoteFileCore.cpp`, `EmoteMetadata.cpp`, `EmoteMotion.cpp`, `EmoteNode.cpp`, `EmoteFrame.cpp`, `EmoteTimeline.cpp`, `EmotePhysics.cpp`
>
> Legend:
> - `✅`: Explicitly read/used by the code; meaning is relatively certain
> - `⚠️`: Can be inferred, but not completely certain
> - Blank purpose: Currently unknown, or unused by the current implementation

---

## 1. Root-level fields

| Field | Type | Purpose | Certainty / Notes |
|---|---|---|---|
| `id` | string |  | Not observed being used by the current C++ parser |
| `label` | string |  | Not observed being used by the current C++ parser |
| `metadata` | object | Master table for character/controller/variable/import information | ✅ Parsed by `emotemetadata` |
| `object` | object | Character object collection, where each object contains a motion tree | ✅ |
| `screenSize` | object | Logical drawing canvas size and origin | ✅ |
| `source` | object | Layer texture resource definitions (icons/textures) | ✅ |
| `spec` | string | Resource specification: `krkr` / `win` / `common` | ✅ Used to determine parsing branch |
| `stereovisionProfile` | object | Stereoscopic vision parameters | ✅ Read, but barely used by the current rendering path |
| `version` | number |  | Not observed being used by the current C++ parser |

---

## 2. `screenSize`

| Field | Type | Purpose | Certainty / Notes |
|---|---|---|---|
| `originX` | int | Logical coordinate origin X | ✅ |
| `originY` | int | Logical coordinate origin Y | ✅ |
| `width` | int | Logical canvas width | ✅ |
| `height` | int | Logical canvas height | ✅ |

---

## 3. `stereovisionProfile`

| Field | Type | Purpose | Certainty / Notes |
|---|---|---|---|
| `dist_e2d` | number |  | Read, but purpose in the current path is unclear |
| `dist_eye` | number |  | Read, but purpose in the current path is unclear |
| `eye_angle_ltd` | number |  | Read, but purpose in the current path is unclear |
| `f_level` | number |  | Read, but purpose in the current path is unclear |
| `fov` | number |  | Read, but purpose in the current path is unclear |
| `len_disp` | number |  | Read, but purpose in the current path is unclear |

---

## 4. `metadata` (top level)

### 4.1 Fields directly read by the current implementation

| Field | Type | Purpose | Certainty / Notes |
|---|---|---|---|
| `base` | object | Default character and default motion | ✅ |
| `base.chara` | string | Default character object name | ✅ |
| `base.motion` | string | Default motion name | ✅ |
| `mirror` | int | Mirror switch (`1` means mirrored) | ✅ |
| `timelineControl` | array | Collection of timeline variable controls | ✅ |
| `variableList` | array | Controllable variable definitions, at least including labels | ✅ |
| `eyeControl` | array | Blink control parameters | ✅ |
| `eyebrowControl` | array | Eyebrow control parameters | ✅ |
| `bustControl` | array | Bust physics control parameters | ✅ |
| `hairControl` | array | Hair physics control parameters | ✅ |
| `partsControl` | array | Physics control parameters for other parts | ✅ |
| `selectorControl` | array | Option/selector switching control | ✅ Empty in this sample |
| `attrcomp` | array | Attribute removal rules | ✅ Field may be absent |

### 4.2 Fields present in this sample but not directly consumed, or only partially consumed, by the current implementation

| Field | Type | Purpose | Certainty / Notes |
|---|---|---|---|
| `catalog` | array | ⚠️ Variable presets; in the sample these are variable-assignment script strings | Inferred |
| `clampControl` | array | ⚠️ Variable clamping/limit rules (`min/max`) | Inferred; current code does not directly execute them |
| `customPartsOrder` | array |  | Not observed being used by the current code |
| `editLock` | int |  | Not observed being used by the current code |
| `format` | string |  | Not observed being used by the current code |
| `instantVariableList` | array | ⚠️ Instant variable list | Inferred |
| `loopControl` | array |  | Empty in this sample; not observed being used by the current code |
| `mirrorControl` | object | ⚠️ Variable matching rules for mirroring (`variableMatchList`) | Code contains comments, but does not actually apply them |
| `mouthControl` | array | ⚠️ Mouth/lip-sync control (`talkLabel`) | Inferred; current parser does not wire this in |
| `psdGlobalImportInfo` | object | PSD import metadata | Not observed being used by the current code |
| `psdLayoutImportInfoMap` | object | PSD layer-layout import mapping | Not observed being used by the current code |
| `psdScrapbookImportInfoMap` | object | PSD scrapbook/parts import mapping | Not observed being used by the current code |
| `psdTextureImportInfoMap` | object | PSD texture import mapping | Not observed being used by the current code |
| `scale` | int |  | Not observed being used by the current code |
| `stereovisionControl` | object | ⚠️ Stereoscopic variable matching rules | Inferred; current parser does not wire this in |
| `transitionControl` | array | ⚠️ Transition switch list | Inferred; current parser does not wire this in |
| `version` | int |  | Not observed being used by the current code |

---

## 5. `metadata` substructures

### 5.1 `timelineControl[]`

| Field | Type | Purpose | Certainty / Notes |
|---|---|---|---|
| `label` | string | Timeline name | ✅ |
| `lastTime` | int | Timeline duration / final frame time | ✅ |
| `loopBegin` | int | Loop start | ✅ |
| `loopEnd` | int | Loop end | ✅ |
| `diff` | int | ⚠️ Differential timeline flag | Inferred; code reads it as `int8_t` |
| `variableList` | array | Variable tracks on the timeline | ✅ |

`timelineControl[].variableList[]`:

| Field | Type | Purpose | Certainty / Notes |
|---|---|---|---|
| `label` | string | Variable name | ✅ |
| `frameList` | array | Variable keyframes | ✅ |

`timelineControl[].variableList[].frameList[]`:

| Field | Type | Purpose | Certainty / Notes |
|---|---|---|---|
| `time` | number | Frame time | ✅ |
| `type` | int | Frame type | ✅ |
| `content.easing` | number | Interpolation/easing parameter | ✅ |
| `content.value` | number | Variable value | ✅ |

### 5.2 `variableList[]` (metadata level)

| Field | Type | Purpose | Certainty / Notes |
|---|---|---|---|
| `label` | string | Variable name | ✅ |
| `frameList` | array |  | Current implementation does not consume internal frame data |
| `frameList[].label` | string |  | Not read by the current implementation |
| `frameList[].frame` | number |  | Not read by the current implementation |

### 5.3 `eyeControl[]`

| Field | Type | Purpose | Certainty / Notes |
|---|---|---|---|
| `label` | string | Controlled variable name | ✅ |
| `beginFrame` | int | Start frame | ✅ |
| `endFrame` | int | End frame | ✅ |
| `blinkFrameCount` | int | Number of frames in one blink | ✅ |
| `blinkIntervalMin` | int | Minimum interval between blinks | ✅ |
| `blinkIntervalMax` | int | Maximum interval between blinks | ✅ |
| `blinkEnabled` | int/bool | ⚠️ Whether blinking is enabled | Inferred; current code does not directly read this key |
| `enabled` | int/bool |  | Current code does not directly read this key |
| `edge` | number |  | Current code does not directly read this key |
| `node` | string |  | Current code does not directly read this key |
| `parameter` | string |  | Current code does not directly read this key |

### 5.4 `eyebrowControl[]`

| Field | Type | Purpose | Certainty / Notes |
|---|---|---|---|
| `label` | string | Controlled variable name | ✅ |
| `beginFrame` | int | Start frame | ✅ |
| `endFrame` | int | End frame; defaults to `beginFrame` when omitted | ✅ |
| `enabled` | int/bool |  | Current code does not directly read this key |
| `edge` | number |  | Current code does not directly read this key |
| `node` | string |  | Current code does not directly read this key |

### 5.5 `bustControl[]`

| Field | Type | Purpose | Certainty / Notes |
|---|---|---|---|
| `label` | string | Controller label | ✅ |
| `var_lr` | string | Left/right control variable | ✅ |
| `var_ud` | string | Up/down control variable | ✅ |
| `friction` | number | Friction parameter | ✅ |
| `gravity` | number | Gravity parameter | ✅ |
| `spring` | number | Spring/elasticity parameter | ✅ |
| `scale_x` | number | X scale parameter | ✅ |
| `scale_y` | number | Y scale parameter | ✅ |
| `param` | object | Initial physics state | ✅ |
| `param.ofs` | number | Offset | ✅ |
| `param.op.{x,y,z}` | number |  | Read by the code; exact physics semantics not fully confirmed |
| `param.p.{x,y,z}` | number |  | Same as above |
| `param.pv.{x,y,z}` | number |  | Same as above |
| `baseLayer` | string | ⚠️ Target/base layer name | Inferred |
| `enabled` | int/bool | ⚠️ Enable switch | Inferred |
| `parameter` | string |  | Unconfirmed |

### 5.6 `hairControl[]` / `partsControl[]`

Both use the same structure, parsed by `uniControl`:

| Field | Type | Purpose | Certainty / Notes |
|---|---|---|---|
| `label` | string | Controller label | ✅ |
| `var_lr` | string | Left/right variable | ✅ |
| `var_lrm` | string |  | Read by the code; semantics not fully confirmed |
| `var_ud` | string | Up/down variable | ✅ |
| `b_rate` | number |  | Read by the code; exact meaning not fully confirmed |
| `bend_spd` | number |  | Same |
| `bend_vol` | number |  | Same |
| `friction_x` | number |  | Same |
| `friction_y` | number |  | Same |
| `gravity` | number | Gravity parameter | ✅ Read |
| `ud_eft` | number |  | Read by the code; semantics not fully confirmed |
| `v_bound` | number |  | Read by the code; semantics not fully confirmed |
| `scale_x` | array[number] | Per-segment X scale | ✅ |
| `scale_y` | array[number] | Per-segment Y scale | ✅ |
| `length` | array[number] | Per-segment length | ✅ |
| `param` | object | Per-segment physics parameters | ✅ |
| `param.ofs` | number | Offset | ✅ |
| `param.bendR` | number |  | Read by the code; semantics not fully confirmed |
| `param.bendS` | number |  | Read by the code; semantics not fully confirmed |
| `param.op` | object |  | Read by the code; semantics not fully confirmed |
| `param.bp[]` | array[object] | Baseline/reference point chain | ⚠️ Inferred; used by the code for segment initialization |
| `param.p[]` | array[object] | Current-position chain | ⚠️ Inferred |
| `param.pv[]` | array[object] | Velocity chain | ⚠️ Inferred |
| `baseLayer` | string | ⚠️ Target/base layer name | Inferred |
| `enabled` | int/bool | ⚠️ Enable switch | Inferred |
| `parameter` | string |  | Unconfirmed |

---

## 6. `object` structure

`object` is a map whose keys are object names such as `all_parts` and `head_parts`.

Each `object.<name>` has the following structure:

| Field | Type | Purpose | Certainty / Notes |
|---|---|---|---|
| `type` | int | Object type | ✅ |
| `metadata` | object/null |  | Current implementation does not use it |
| `motion` | object | Motion collection, keyed by motion name | ✅ |

---

## 7. `object.<name>.motion.<motionName>`

| Field | Type | Purpose | Certainty / Notes |
|---|---|---|---|
| `type` | int | Motion type | ✅ |
| `lastTime` | int | Motion end time | ✅ |
| `loopTime` | int | Loop period; `-1` commonly appears to mean no looping | ⚠️ Inferred |
| `layer` | array | Root node list | ✅ |
| `priority` | array | Rendering priority list | ✅ Reads `content` |
| `parameter` | array | Parameter definitions | ✅ |
| `parameterize` | int/null | Parameterization flag/index | ✅ |
| `variable` | array |  | Current implementation does not use it |
| `metadata` | object/null |  | Current implementation does not use it |
| `referenceModelFileList` | array |  | Current implementation does not use it |
| `referenceProjectFileList` | array |  | Current implementation does not use it |
| `tag` | array |  | Current implementation does not use it |

`motion.parameter[]`:

| Field | Type | Purpose | Certainty / Notes |
|---|---|---|---|
| `id` | string | Parameter name | ✅ |
| `rangeBegin` | number | Range start | ✅ |
| `rangeEnd` | number | Range end | ✅ |
| `division` | int | Number of discrete subdivisions | ✅ |
| `enabled` | int/bool | ⚠️ Enabled flag | Inferred |
| `discretization` | int |  | Current code does not use this value |

`motion.priority[]`:

| Field | Type | Purpose | Certainty / Notes |
|---|---|---|---|
| `content` | array[int] | Node-index ordering list | ✅ |
| `time` | number |  | Current implementation does not use it |
| `type` | int |  | Current implementation does not use it |

---

## 8. `layer` nodes (`motion.layer[]` and their `children[]`)

> **For pose and world-coordinate implementation:** see [`MOTIONPLAYER_TEXTURE_WORLD_COORDS.md`](MOTIONPLAYER_TEXTURE_WORLD_COORDS.md), covering the `coord` / `blank` / `source.origin` / `progress` matrix stack.

| Field | Type | Purpose | Certainty / Notes |
|---|---|---|---|
| `label` | string | Node name, usable for lookup | ✅ |
| `type` | int | Node type | ✅ |
| `frameList` | array | Node keyframes | ✅ |
| `children` | array | Child nodes | ✅ |
| `inheritMask` | int | Inheritance/transformation mask | ✅ Used for behavior branching |
| `meshCombine` | int | Mesh-combination strategy | ⚠️ Inferred; field is read |
| `meshDivision` | int | Mesh subdivision level | ✅ Used for CPU-side mesh subdivision |
| `meshTransform` | int | Mesh transformation mode | ⚠️ Inferred; field is read |
| `meshSyncChildMask` | int | Child-node mesh synchronization mask | ⚠️ Inferred |
| `parameterize` | int/null | Parameterization index | ✅ Participates in variable-driven behavior when present |
| `stencilCompositeMaskLayerList` | array[string] | Associated layer names for stencil compositing | ✅ |
| `objTriPriority` | int |  | Current implementation does not use it |
| `stencilType` | int |  | Current implementation does not use it |
| `coordinate` | int |  | Field exists; current implementation does not directly read it |
| `groundCorrection` | int |  | Current implementation does not directly read it |
| `joinTarget` | string/null |  | Current implementation does not directly read it |
| `metadata` | object/null |  | Current implementation does not directly read it |
| `transformOrder` | int |  | Current implementation does not directly read it |
| `exportSelf` | int/bool |  | Current implementation does not directly read it |

---

## 9. `frameList[]` and `content`

`frameList[]`:

| Field | Type | Purpose | Certainty / Notes |
|---|---|---|---|
| `time` | number | Frame time | ✅ |
| `type` | int | Frame type | ✅ |
| `content` | object | Frame content; may be empty | ✅ |

`content` fields observed in this sample:

| Field | Type | Purpose | Certainty / Notes |
|---|---|---|---|
| `src` | string | Source reference, either an icon or a child motion/layout | ✅ |
| `mask` | int | Stencil/mask flag | ✅ Read |
| `bm` | int | Blend mode | ✅ Mapped to blit mode |
| `ox` | number | X offset | ✅ |
| `oy` | number | Y offset | ✅ |
| `coord` | array[number] | 3D coordinates (x/y/z) | ✅ |
| `angle` | number | Rotation angle | ✅ Supported field; not widely present in this sample |
| `sx` | number | X shear/skew | ✅ Supported field |
| `sy` | number | Y shear/skew | ✅ Supported field |
| `zx` | number | X scale | ✅ Supported field |
| `zy` | number | Y scale | ✅ Supported field |
| `opa` | number | Opacity | ✅ Supported field |
| `zcc` | object | Z-related curve control | ⚠️ Inferred; field is read |
| `ccc` | object | Color/curve control | ⚠️ Inferred; field is read |
| `mesh` | object | Mesh deformation data | ✅ |
| `motion` | object |  | Supported by an older-format parsing branch; rare in this sample |
| `icon` | string |  | Supported by an older-format parsing branch; rare in this sample |

`content.mesh`:

| Field | Type | Purpose | Certainty / Notes |
|---|---|---|---|
| `bp` | array[32]/null | Mesh control offsets, 32 points | ✅ |
| `cc` | object/null | Curve-control parameters | ⚠️ Inferred |
| `cc.c` | array[2] |  | Read; semantics not fully confirmed |
| `cc.x` | array[4] |  | Read; semantics not fully confirmed |
| `cc.y` | array[4] |  | Read; semantics not fully confirmed |

`content.zcc` / `content.ccc`:

| Field | Type | Purpose | Certainty / Notes |
|---|---|---|---|
| `c` | array[2] |  | Read; semantics not fully confirmed |
| `x` | array[4] |  | Read; semantics not fully confirmed |
| `y` | array[4] |  | Read; semantics not fully confirmed |

---

## 10. `source` structure

`source` is a map whose keys are resource-group names such as `face_mouth`.

Each `source.<name>`:

| Field | Type | Purpose | Certainty / Notes |
|---|---|---|---|
| `type` | int | Resource type | ✅ |
| `icon` | object | Collection of image variants | ✅ |
| `metadata` | object/null |  | Current implementation does not use it |

`source.<name>.icon` is a map using variant identifiers such as `"1"`, `"3"`, etc.:

| Field | Type | Purpose | Certainty / Notes |
|---|---|---|---|
| `originX` | number | Texture anchor X | ✅ |
| `originY` | number | Texture anchor Y | ✅ |
| `width` | number | Image tile width | ✅ |
| `height` | number | Image tile height | ✅ |
| `compress` | string | Compression type; `RL` in this sample | ✅ |
| `pixel` | resource/object | Pixel resource reference | ✅ |
| `pal` | resource/object | Palette resource, if present | ✅ Conditionally read |
| `clip` | object | Clipping rectangle | ✅ |
| `metadata` | object/null |  | Current implementation does not use it |
| `attr` | object |  | Field exists; current implementation does not use it |

`icon.clip`:

| Field | Type | Purpose | Certainty / Notes |
|---|---|---|---|
| `left` | number | Clip left | ✅ |
| `top` | number | Clip top | ✅ |
| `right` | number | Clip right | ✅ |
| `bottom` | number | Clip bottom | ✅ |

`pixel`, a PSB resource reference object, has the following structure in this sample:

| Field | Type | Purpose | Certainty / Notes |
|---|---|---|---|
| `__psb` | string | PSB resource identifier | ⚠️ Inferred |
| `index` | int | Resource index | ✅ Used by `parseResource` |
| `bytes` | int | Resource byte size | ⚠️ Inferred |
| `extra` | bool |  | Meaning unconfirmed |

---

## 11. PSD-import-related fields

These fields are present only as sample metadata. They do not directly participate in rendering or physics updates in the current `motionplayer` runtime path.

### 11.1 `psdGlobalImportInfo`

| Field | Type | Purpose | Certainty / Notes |
|---|---|---|---|
| `layout` | int |  | Meaning unconfirmed |
| `layoutZ` | int |  | Meaning unconfirmed |

### 11.2 `psdLayoutImportInfoMap.<key>`

| Field | Type | Purpose | Certainty / Notes |
|---|---|---|---|
| `psdLabel` | string | ⚠️ Corresponding PSD layer name | Inferred |
| `z` | int | ⚠️ Layer Z order | Inferred |

### 11.3 `psdScrapbookImportInfoMap.<key>`

| Field | Type | Purpose | Certainty / Notes |
|---|---|---|---|
| `psdLabel` | string | ⚠️ PSD layer name | Inferred |
| `blendMode` | int | ⚠️ Blend mode | Inferred |
| `mask` | int | ⚠️ Whether this is a mask | Inferred |
| `remove` | int | ⚠️ Whether it should be removed | Inferred |
| `z` | int | ⚠️ Z order | Inferred |
| `clipRect` | object | ⚠️ Clipping region | Inferred |
| `clipRect.left/top/width/height` | number | ⚠️ Clipping-rectangle parameters | Inferred |

### 11.4 `psdTextureImportInfoMap.<key>`

| Field | Type | Purpose | Certainty / Notes |
|---|---|---|---|
| `blendMode` | int | ⚠️ Blend mode | Inferred |
| `framePsdLabelList` | array[string] | ⚠️ Frame-layer list | Inferred |
| `iconPsdLabelMap` | object | ⚠️ Mapping from icon names to PSD layer names | Inferred |
| `isValid` | int/bool | ⚠️ Whether valid | Inferred |
| `mask` | int | ⚠️ Mask flag | Inferred |
| `remove` | int | ⚠️ Removal flag | Inferred |
| `z` | int | ⚠️ Z order | Inferred |

---

## 12. Fields known to be incompletely supported or unused by the current implementation

The following fields exist in the JSON but are not fully implemented by the current code, either because they are only read or not read at all:

- `metadata.mirrorControl.variableMatchList` — code contains comments, but the feature is not enabled
- `metadata.transitionControl`
- `metadata.clampControl`
- `metadata.mouthControl`
- `metadata.psd*` import-mapping fields
- `layer.transformOrder`, `layer.coordinate`, `layer.exportSelf`, `layer.groundCorrection`, `layer.joinTarget`
- `motion.variable`, `motion.tag`, `motion.metadata`
- Some older-format keys under `frame.content`, such as `motion` and `icon`

---

## 13. Notes

- This document aligns **one specific JSON sample** with the **current repository implementation**. It should not be treated as a complete specification of the e-mote format.
- If a strict specification is needed later, it would be advisable to compare against upstream e-mote SDK documentation and additional samples using the `win` and `common` specifications.