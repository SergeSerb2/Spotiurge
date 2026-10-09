# Root confirmation of the two final design corrections

Opus round 2 confirmed the five original findings and requested two bounded
corrections. Its disposition was FIX with permission for root to confirm the
three affected screens, without a new delegate or full capture cycle.

Root added the Form's explicit hierarchical foreground styles and changed the
dark disabled CTA fill to the desktop's white overlay at 8%. On that lighter
neutral surface, disabled text uses the existing dim ink so it stays readable.
Light preserves its established disabled palette. Native disabled semantics
remain in force.

The Simulator was rebuilt, ad-hoc signed and recaptured only for
`settings-light`, `settings-demo-light` and `onboarding-dark`. The reviewer's
read-only pixel sampler was rerun against the native PNGs:

| Role | Sampled ratio |
| --- | --- |
| Light "Not in this build" | 17.58:1 |
| Light "Demo: no audio" | 17.58:1 |
| Dark disabled "Write your taste" | 5.67:1 |
| Unchanged light disabled CTA | 3.73:1 |

The dark pill is now visibly separate from its content plate; the source uses
an 8% white overlay rather than an opaque surface-active fill. Root viewed the
fresh onboarding and Settings captures. No new material issue was observed.
All prior limitations in the Opus verdict remain, including enabled feedback
pixels, live data/audio, motion and accessibility walkthroughs.

This is root's closure of the named findings, not a new overall SHIP verdict
from Opus. Per-capture build hashes distinguish the original 23 reviewed r3
frames from the final three corrections.
