// This file was generated by [ts-rs](https://github.com/Aleph-Alpha/ts-rs). Do not edit this file manually.
import type { AttributeValueSpec } from "./AttributeValueSpec";
import type { ComponentSpecVariant } from "./ComponentSpecVariant";
import type { PositionSpec } from "./PositionSpec";

export type ComponentSpec = { name: string, position: PositionSpec, variant: ComponentSpecVariant, needsDestroy: boolean, deletionUserPk: string | null, uniqueId: string, deleted: boolean, attributes: Array<AttributeValueSpec>, inputSockets: Array<AttributeValueSpec>, outputSockets: Array<AttributeValueSpec>, };
