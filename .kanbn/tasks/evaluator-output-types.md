---
created: 2023-09-20T13:46:47.471Z
updated: 2023-09-20T13:46:47.468Z
assigned: ""
progress: 0
tags:
- Architecture
---

# Evaluator Output Types

Right now, evaluator's output type must be the same as the type read from kdl. Instead, an evaluator should have a parsed type (bool, int, float, string) and an output type (same as parsed or impls From/FromStr of the underlying type). This would support something like `Value<Rest>` instead of needing to use `Rest::from_str(Value<String>::evaluate())`. The output type can be lazily provided at evaluation time, instead of being a compile-time restriction. Heck, at that rate we dont even need a compile-time parsed type because it can be stored as a KdlValue (enum wrapper around all primitive types), which is then parsed at evaluation time. This goes back to the initial implementation debate of is it better to ensure the input data is accurate at parse time or eval time, and given the pain it has been to create non-primitive typed evals, I think i'd be better to find a middle ground than going to one extreme or the other. In that world, the parsed and output types don't matter at compile time, but when the evaluator parses kdl and is evaluated, both of those functions should take the expected type so that the kdl can be checked at parse time, stored as a general primitive, and then parsed again at eval time.
