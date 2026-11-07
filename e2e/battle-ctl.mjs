// The battle's controls on the table (arena#29): on the result (arena#53) the control bar's ↻ cell is `end-replay`;
// with the result put away it is `battle-replay`.
export async function replayBattle(page) {
  if (await page.getByTestId("end-card").isVisible().catch(() => false)) await page.getByTestId("end-replay").click();
  else await page.getByTestId("battle-replay").click();
}
