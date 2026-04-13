<script lang="ts">
  import OnboardingStep1 from "./OnboardingStep1.svelte";
  import OnboardingStep2 from "./OnboardingStep2.svelte";
  import OnboardingStep3 from "./OnboardingStep3.svelte";
  import OnboardingStep4 from "./OnboardingStep4.svelte";

  interface Props {
    onComplete: (prefixKey: string) => void;
  }

  let { onComplete }: Props = $props();

  let step = $state(1);
  let chosenPrefixKey = $state("ctrl+b");

  function handleStep1Complete(prefixKey: string) {
    chosenPrefixKey = prefixKey;
    step = 2;
  }

  function handleStep1Skip() {
    chosenPrefixKey = "ctrl+b";
    onComplete(chosenPrefixKey);
  }

  function handleStep2Next(_shell: string) {
    step = 3;
  }

  function handleStep2Skip() {
    step = 3;
  }

  function handleStep3Next() {
    step = 4;
  }

  function handleStep3Skip() {
    step = 4;
  }

  function handleStep4Start() {
    onComplete(chosenPrefixKey);
  }
</script>

<div class="overlay">
  <div class="panel">
    {#if step === 1}
      <OnboardingStep1
        onComplete={handleStep1Complete}
      />
    {:else if step === 2}
      <button class="close-btn" onclick={handleStep1Skip} aria-label="Close">✕</button>
      <OnboardingStep2
        onNext={handleStep2Next}
        onBack={() => (step = 1)}
        onSkip={handleStep2Skip}
      />
    {:else if step === 3}
      <button class="close-btn" onclick={handleStep1Skip} aria-label="Close">✕</button>
      <OnboardingStep3
        onNext={handleStep3Next}
        onBack={() => (step = 2)}
        onSkip={handleStep3Skip}
      />
    {:else if step === 4}
      <OnboardingStep4
        prefixKey={chosenPrefixKey}
        onStart={handleStep4Start}
        onBack={() => (step = 3)}
      />
    {/if}
  </div>
</div>

<style>
  .overlay {
    position: fixed;
    inset: 0;
    background: rgba(0, 0, 0, 0.85);
    display: flex;
    align-items: center;
    justify-content: center;
    z-index: 500;
    font-family: system-ui, -apple-system, sans-serif;
  }

  .panel {
    background: #1a1a1a;
    border: 1px solid #333;
    border-radius: 12px;
    width: 520px;
    max-width: 92vw;
    box-shadow: 0 16px 48px rgba(0, 0, 0, 0.7);
    position: relative;
    display: flex;
    flex-direction: column;
    overflow: hidden;
    min-height: 380px;
  }

  .close-btn {
    position: absolute;
    top: 0.75rem;
    right: 0.75rem;
    background: none;
    border: none;
    color: #555;
    cursor: pointer;
    font-size: 0.85rem;
    padding: 0.2rem 0.4rem;
    border-radius: 4px;
    z-index: 1;
  }

  .close-btn:hover {
    color: #d9d4c7;
    background: #2a2a2a;
  }
</style>
