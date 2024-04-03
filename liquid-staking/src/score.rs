multiversx_sc::imports!();
use super::{common, constants::*, events, model::*, proxies, storage};

#[multiversx_sc::module]
pub trait ScoreModule:
    admin::AdminModule + common::CommonModule + events::EventsModule + proxies::ProxyModule + storage::StorageModule
{
    fn compute_delegation_score(&self, contract_data: &DelegationContractData<Self::Api>) -> BigUint {
        self.compute_delegation_score_internal(&contract_data.total_value_locked, &contract_data.apr)
    }

    /// The Delegation Score is given by the weighted average between the Total Value Locked score and the Annual
    /// Percentage Rate score.
    ///
    fn compute_delegation_score_internal(&self, total_value_locked: &BigUint, apr: &BigUint) -> BigUint {
        let model = self.delegation_score_model().get();
        let DelegationScoreModel {
            method,
            min_tvl,
            max_tvl,
            min_apr,
            max_apr,
            omega,
        } = model;

        match method {
            DelegationScoreMethod::Tvl => self.compute_tvl_score(total_value_locked, &min_tvl, &max_tvl),
            DelegationScoreMethod::Apr => self.compute_apr_score(apr, &min_apr, &max_apr),
            DelegationScoreMethod::Mixed => {
                let bps = BigUint::from(BPS);
                let tvl_score = self.compute_tvl_score(total_value_locked, &min_tvl, &max_tvl);
                let apr_score = self.compute_apr_score(apr, &min_apr, &max_apr);
                (&omega * &tvl_score + (&bps - &omega) * &apr_score) / &bps
            },
        }
    }

    /// Computes the Total Value Locked (TVL) score, in which lower TVLs yield higher scores. The score is capped at one
    /// at low TVLs and floored at zero at high TVLs.
    ///
    fn compute_tvl_score(&self, total_value_locked: &BigUint, min_tvl: &BigUint, max_tvl: &BigUint) -> BigUint {
        self.norm_linear_clamp(total_value_locked, min_tvl, max_tvl, true)
    }

    /// Computes the Annual Percentage Rate (APR) score, in which higher APRs yield higher scores. The score is capped
    /// at one at high APRs and floored at zero at low TVLs.
    ///
    fn compute_apr_score(&self, apr: &BigUint, min_apr: &BigUint, max_apr: &BigUint) -> BigUint {
        self.norm_linear_clamp(apr, min_apr, max_apr, false)
    }
}
