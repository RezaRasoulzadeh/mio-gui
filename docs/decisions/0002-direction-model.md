# 0002 — Direction Model

Status: accepted

RTL and LTR are equal, cascading layout directions. Direction can be inherited and overridden by any subtree.

Layout and components use logical start/end concepts. The layout layer resolves logical geometry into physical coordinates before submission to direction-neutral rendering primitives.

Text base direction remains distinct from container layout direction so mixed-direction content can be resolved correctly by the Unicode bidi engine.
