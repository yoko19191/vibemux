<script lang="ts">
  interface Props {
    color: string;
  }

  let { color }: Props = $props();
</script>

<span class="busy-indicator" style="border-color: {color};"></span>

<style>
  .busy-indicator {
    display: inline-block;
    width: 10px;
    height: 10px;
    border: 1.5px solid;
    border-radius: 1px;
    flex-shrink: 0;
    animation: spin 1s linear infinite;
  }

  @keyframes spin {
    from { transform: rotate(0deg); }
    to { transform: rotate(360deg); }
  }
</style>
