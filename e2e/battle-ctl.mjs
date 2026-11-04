// The battle's controls on the table (arena#29): the result covers the tray, so a Replay from a
// finished battle is the result's own button; with the result put away it is the control bar's ↻.
export async function replayBattle(page) {
  if (await page.getByTestId("end-card").isVisible().catch(() => false)) await page.getByTestId("end-replay").click();
  else await page.getByTestId("battle-replay").click();
}
