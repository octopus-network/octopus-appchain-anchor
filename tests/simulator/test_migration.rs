use near_sdk::json_types::U64;
use near_sdk_sim::call;

use crate::common;

#[test]
fn test_migration() {
    let user0_id_in_appchain =
        "0xd43593c715fdd31c61141abd04a99fd6822c8558854ccde39a5684e7a56da27d".to_string();
    let user1_id_in_appchain =
        "d43593c715fdd31c61141abd04a99fd6822c8558854ccde39a5684e7a56da270".to_string();
    //
    let (root, _, _, _, anchor, users, _) = common::test_normal_actions(true, false);
    common::deploy_new_anchor_contract(&anchor);
    let result = call!(root, anchor.migrate_state());
    common::print_execution_result("migrate_state", &result);
    //
    //
    //
    common::print_anchor_status(&anchor);
    common::print_wrapped_appchain_token_info(&anchor);
    common::print_appchain_settings(&anchor);
    common::print_anchor_settings(&anchor);
    common::print_validator_set_info_of(&anchor, U64::from(0));
    common::print_validator_list_of(&anchor, Some(0));
    common::print_validator_list_of(&anchor, Some(1));
    common::print_validator_list_of(&anchor, Some(2));
    common::print_validator_list_of(&anchor, Some(3));
    common::print_user_staking_histories_of(&anchor, &users[0]);
    common::print_user_staking_histories_of(&anchor, &users[1]);
    common::print_user_staking_histories_of(&anchor, &users[2]);
    common::print_user_staking_histories_of(&anchor, &users[3]);
    common::print_user_staking_histories_of(&anchor, &users[4]);
    common::print_validator_profile(&anchor, &users[0].account_id(), &user0_id_in_appchain);
    common::print_validator_profile(&anchor, &users[1].account_id(), &user1_id_in_appchain);
    common::print_staking_histories(&anchor);
    common::print_anchor_events(&anchor);
    common::print_appchain_notifications(&anchor);
}
