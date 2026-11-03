# Research 03 — Temporal Consistency, Partial-Order Reconstruction, and Formal Certification of Asynchronous Financial Data

## 1. Scientific Scope

This research investigates the reconstruction and certification of temporal precedence under bounded timestamp uncertainty and explicit event-order constraints.

The primary object is not a price forecast, a market-state estimator, or a conventional point-in-time data record. It is the set of temporal configurations compatible with the evidence available for a finite collection of asynchronous observations.

The study addresses three questions:

1. Is the supplied temporal evidence jointly feasible?
2. Which precedence relations hold in every feasible reconstruction, and which remain ambiguous?
3. How do certified precedence relations change as timestamp uncertainty increases?

The implementation is intended as reproducible computational research, not a production-grade clock-synchronization service or a claim of new temporal-constraint theory.

## 2. Relationship to Existing Quassar Research

Research 01 in quassar-data establishes validated financial observations with observation, publication, and availability timestamps. Research 03 does not replace those semantics. It investigates uncertain event-time reconstruction, which is distinct from checking whether an already timestamped observation was available at a historical decision time.

Research 02 in quassar-data concerns finite-field arithmetic and algebraic error-correcting codes. Research 03 does not perform algebraic decoding.

The scope excludes portfolio risk, latent factor estimation, regime switching, market microstructure dynamics, Bayesian event detection, and generic uncertainty quantification. Its scientific object is the feasibility and identifiability of temporal order under explicit constraints.

## 3. Mathematical Model

Let E = {1, ..., n} be a finite event set. Each event i has an unknown real occurrence time t_i.

A timestamp measurement s_i and a nonnegative uncertainty bound epsilon_i define

    s_i - epsilon_i <= t_i <= s_i + epsilon_i.

More generally, each event may have a finite interval [l_i, u_i], with l_i <= u_i.

Known ordering and timing information is represented by difference constraints

    t_j - t_i <= c_ij,

where c_ij is finite.

An asserted non-strict precedence i before-or-at j is expressed as

    t_i - t_j <= 0.

A strict precedence assertion requires an explicit positive separation delta:

    t_i - t_j <= -delta, with delta > 0.

This separation is a modeling assumption, not a substitute for an exact open inequality. The implementation will distinguish strict order in the inferred result from any user-supplied minimum-separation constraint.

The feasible temporal set is

    F = {t in R^n : l_i <= t_i <= u_i and
         t_j - t_i <= c_ij for every declared constraint}.

A timestamp interval describes uncertainty in occurrence time. It does not establish publication time, ingestion time, data availability, or causal influence.

## 4. Difference-Constraint Representation

Introduce a reference variable t_0 = 0. Encode each bound and relative-time constraint as a directed weighted edge representing

    t_v <= t_u + w(u, v).

The interval bounds become

    t_i - t_0 <= u_i,
    t_0 - t_i <= -l_i.

The model is feasible if and only if its finite difference-constraint graph has no negative-weight directed cycle.

A negative cycle is a certificate of inconsistency: summing its inequalities yields 0 <= W for a strictly negative total weight W.

For a feasible graph, shortest-path closure provides the tightest implied upper bounds on differences between reachable variables. If d(i, j) is the shortest-path distance from i to j, then

    t_j - t_i <= d(i, j).

When no path exists, this bound is not finite. The implementation must not interpret an unreachable path as a zero bound.

All event variables have finite input intervals in the initial research scope, so the augmented graph provides finite bounds for every event pair whenever the model is feasible.

## 5. Certified Precedence and Ambiguity

For a feasible model, define the tight bounds on the difference t_j - t_i:

    U_ij = d(i, j),
    L_ij = -d(j, i).

The relation i necessarily precedes j in strict temporal order exactly when

    U_ij < 0.

The relation j necessarily precedes i exactly when

    L_ij > 0.

If neither condition holds, strict order is not identifiable from the constraints. This classification includes feasible equality and, depending on the bounds, feasible realizations of different orderings.

The research will distinguish:

- Necessary strict precedence: t_i < t_j in every feasible reconstruction.
- Necessary equality: t_i = t_j in every feasible reconstruction.
- Order ambiguity: neither strict direction is necessary.
- Inconsistency: no feasible reconstruction exists.

The classification must never convert a non-strict constraint into strict precedence without a mathematically justified margin.

A negative shortest-path bound certifies necessary strict precedence. A claim that strict precedence is not necessary requires a feasible reconstruction with t_i >= t_j; such a witness must be independently checkable.

## 6. Independent Certificates

A feasible-instance certificate is an explicit assignment of event times satisfying every original interval and difference constraint.

An inconsistent-instance certificate is a directed negative cycle, accompanied by the constituent input constraints or their traceable representations.

A necessary-precedence certificate is a directed path whose summed weight is strictly negative and which implies the claimed event-time difference.

A non-necessity certificate is a feasible assignment violating the proposed strict precedence.

Certificate verification must operate on the original problem constraints, without trusting the solver's internal classification.

For the reference implementation, integer time units and exact integer arithmetic are preferred to avoid floating-point ambiguity at zero-margin boundaries. Any later floating-point implementation must specify numerical error handling and must not claim exact certification solely from tolerance-based comparisons.

## 7. Uncertainty Sensitivity and Identifiability

For a common uncertainty expansion parameter lambda >= 0, define

    l_i(lambda) = s_i - lambda * epsilon_i,
    u_i(lambda) = s_i + lambda * epsilon_i.

All relative-time constraints remain fixed unless an experiment explicitly states otherwise.

For lambda_1 <= lambda_2, the feasible sets satisfy

    F(lambda_1) subseteq F(lambda_2).

Consequently, provided both sets are nonempty, a strict precedence relation that is necessary at lambda_2 must also be necessary at lambda_1.

The reverse implication need not hold. Expanding uncertainty may destroy a certified ordering but cannot create a new necessary ordering solely by admitting additional temporal configurations.

The research will characterize precedence margins and investigate critical uncertainty levels at which a previously necessary relation becomes ambiguous.

Any computed threshold must be distinguished from an exact mathematical boundary if numerical search or finite precision is used.

## 8. Scientific Claims and Verification Obligations

The implementation must substantiate the following established mathematical properties in its stated model:

P1. Feasibility is equivalent to the absence of a negative directed cycle.

P2. For feasible finite-bound models, shortest-path closure yields the tightest implied pairwise difference bounds.

P3. Necessary strict precedence is equivalent to a strictly negative tight upper bound on the corresponding time difference.

P4. Expanding only the event-time uncertainty intervals cannot create new necessary strict precedence relations between previously feasible configurations.

These are research verification targets and established consequences of difference-constraint theory, not claims of original mathematical discovery.

The investigation will additionally evaluate the practical reliability and computational cost of independently verifiable certificates and the sensitivity of identifiable order to bounded clock uncertainty.

## 9. Experimental Program

Experiment A — Feasibility and inconsistency:
Generate consistent systems from known event-time assignments, introduce controlled contradictory constraints, and independently validate feasible assignments or negative-cycle certificates.

Experiment B — Partial-order recovery:
Construct synthetic event sets with known strict ordering, equality, and ambiguity. Compare certified relations with naive sorting by reported timestamps.

Experiment C — Uncertainty expansion:
Vary lambda over a reproducible grid. Measure the number of necessarily ordered pairs, ambiguity transitions, and violations of the expected monotonicity property.

Experiment D — Boundary and adversarial cases:
Include zero-width intervals, coincident timestamps, exact zero precedence margins, contradictory constraints, disconnected declared-precedence subgraphs, and redundant constraints.

Experiment E — Computational scaling:
Measure execution time, memory use, and certificate size as event and constraint counts increase. Report the hardware, interpreter, dataset generation method, and experimental limitations.

Synthetic data must be labeled as synthetic. No empirical performance advantage or scientific novelty will be asserted before measurement and comparison.

## 10. Implementation Strategy

The initial implementation will use Python 3.11 or later and the standard library.

The core will separate:

1. Validated temporal problem representation.
2. Difference-constraint feasibility and shortest-path computation.
3. Pairwise precedence classification.
4. Independently verifiable certificates.
5. Reproducible numerical experiments.

No third-party runtime dependency or second programming language will be introduced without a demonstrated need.

The existing package, catalog generator, README structure, and CI conventions must remain intact. Research 01 and Research 02 implementations must not be modified merely to accommodate Research 03.

## 11. Research Boundaries

The model assumes correctly specified finite uncertainty intervals and trusted input constraints. It does not infer clock error distributions, recover unknown missing causal links, or establish economic causality.

If input uncertainty bounds are false, a mathematically valid certificate need not describe the real historical event order.

The work does not claim that partial-order reconstruction, difference constraints, negative-cycle detection, or shortest-path closure are novel inventions.

Academic novelty, if claimed in a future publication, requires a specific contribution supported by prior-art comparison, mathematical analysis, and reproducible evidence.

## 12. Foundational References

- Lamport, L. (1978). Time, Clocks, and the Ordering of Events in a Distributed System. Communications of the ACM, 21(7), 558–565.
- Dechter, R., Meiri, I., and Pearl, J. (1991). Temporal Constraint Networks. Artificial Intelligence, 49(1–3), 61–95.
- Bellman, R. (1958). On a Routing Problem. Quarterly of Applied Mathematics, 16(1), 87–90.
- Floyd, R. W. (1962). Algorithm 97: Shortest Path. Communications of the ACM, 5(6), 345.

## 13. Completion Criteria

Research 03 is complete only when:

- The mathematical specification and implementation agree.
- Feasible and infeasible results carry independently checked certificates.
- Necessary precedence and ambiguity are validated against reference cases.
- Uncertainty monotonicity is verified on controlled examples.
- Adversarial and numerical boundary cases pass.
- Experiments are reproducible and report limitations.
- Python lint and tests pass in GitHub Actions.
- The research catalog and generated README index are consistent.
- The pull request is merged, final CI is verified, and the temporary branch is deleted.

No implementation milestone or CI success alone constitutes scientific validation.
