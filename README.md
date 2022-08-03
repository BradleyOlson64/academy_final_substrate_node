# Substrate Node Template

## Layout

In short, the identity pallet feeds information about the valid voter set to the quadratic voting pallet. Then the quadratic voting pallet uses this information to filter voting on proposals. Passed proposals have the potential to call special functionality from the crypto kitties pallet. All of this is soft coupled by the traits living in the brads-soft-coupling dependency. Please feel free to make your own tests to add to mine. The most glaring hole I didn't have time to plug is that the same voter can vote multiple times for the same proposal. The fix would be a simple map with entries for every voter who already voted for the current proposal. This would be cleared after each passed proposal.

### Run
--Test QuadraticVoting pallet cargo test -p quadratic-voting

--Test Identity pallet        cargo test -p identity-pallet

--Test in full node environment: cargo run -r -- --dev

 (I didn't, but feel free to see if this works)

