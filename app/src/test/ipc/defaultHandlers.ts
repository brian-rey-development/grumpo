import type { CommandHandlers } from './routeCommands';
import { populatedDayReport, weekReport, GRANTED_PERMISSIONS } from './usage';
import { DEFAULT_SETTINGS } from './settings';
import { KNOWN_APPS } from './rules';
import { connectedStatus } from './status';

const QUIET_WEEK_SECONDS = [3_600, 7_200, 0, 5_400, 1_800, 0, 0] as const;

/** The reads every screen makes, answered with calm data so that a test overrides only what it checks. */
export function defaultHandlers(): CommandHandlers {
  return {
    get_day_report: ({ date }) => populatedDayReport(date),
    get_week_report: ({ endDate }) => weekReport(endDate, QUIET_WEEK_SECONDS),
    get_limit_usage: () => [],
    get_focus_status: () => connectedStatus({ active: false }),
    get_settings: () => DEFAULT_SETTINGS,
    get_clock_style: () => 'h23',
    get_permissions: () => GRANTED_PERMISSIONS,
    get_app_icon: () => null,
    list_known_apps: () => KNOWN_APPS,
    get_block_screen: () => null,
    get_onboarding_state: () => ({ completed: true }),
  };
}
