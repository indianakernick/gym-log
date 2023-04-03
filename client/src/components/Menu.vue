<script setup lang="ts">
import { getIdGenerator } from '@/utils/id';
import { IonButton, IonContent, IonIcon, IonPopover } from '@ionic/vue';
import { ellipsisHorizontalCircle } from 'ionicons/icons';

defineProps<{
  title: string;
  context: 'title-bar' | 'inline';
  items: {
    title: string;
    handler: () => void;
    theme?: 'default' | 'primary' | 'danger';
    icon?: string;
  }[];
}>();

const id = getIdGenerator('menu')();
</script>

<template>
  <IonButton
    v-if="context === 'title-bar'"
    :id="id"
    :title="title"
  >
    <IonIcon slot="icon-only" :icon="ellipsisHorizontalCircle" />
  </IonButton>

  <button
    v-if="context === 'inline'"
    :id="id"
    :title="title"
  >
    <IonIcon :icon="ellipsisHorizontalCircle" class="block w-6 h-6"/>
  </button>

  <!--
    Unfortunately, the `position: absolute` technique doesn't work inside of
    `ion-header`. There doesn't seem to be any way to escape the ion-header
    using that technique. So we're using `ion-popover`. This does have one minor
    advantage: it will be displayed above the menu button if the button is at
    the bottom of the screen.

    However, the thing I don't like is the animation. It's a rather boring fade.
    I much preferred the scale+fade transition I had before. Customising the
    animation of an `ion-popover` is possible but I haven't been able to get
    even the simplest examples to work. It seems that part of the positioning
    and presentation logic is tied up with the animation. I guess this means
    that you have a lot of power in defining an animation but it also means that
    defining an animation that's independent of the positioning logic isn't
    possible. At least, I'm pretty sure it isn't. Perhaps there's a way to keep
    the default `enter-animation` and `leave-animation` and add some CSS on top
    of that.
  -->

  <IonPopover
    :trigger="id"
    :dismiss-on-select="true"
  >
    <IonContent>
      <ul role="menu" :aria-label="title">
        <li
          v-for="item in items"
          role="presentation"
          class="border-t first:border-none shadow-lg overflow-hidden
            border-neutral-300 bg-neutral-200
            dark:border-neutral-600 dark:bg-neutral-800"
        >
          <!--
            An empty touchstart listener is required to make :active work on iOS
            Safari. This isn't necessary for the modal for some reason.
            https://stackoverflow.com/a/33681490/4093378
          -->
          <button
            role="menuitem"
            @click="item.handler"
            @touchstart.passive=""
            class="w-full py-2 px-3 whitespace-nowrap flex gap-1 items-center
              text-sm active:bg-neutral-300 dark:active:bg-neutral-700"
            :class="{
              'text-blue-500': item.theme === 'primary',
              'text-red-500': item.theme === 'danger',
            }"
          >
            {{ item.title }}
            <IonIcon v-if="item.icon" :icon="item.icon" class="ml-auto w-5 h-5" />
          </button>
        </li>
      </ul>
    </IonContent>
  </IonPopover>
</template>

<style scoped>
ion-popover {
  --backdrop-opacity: 0.5;
  /* neutral-200 from Tailwind. */
  --background: #e5e5e5;
}

@media (prefers-color-scheme: dark) {
  ion-popover {
    /* neutral-800 from Tailwind. */
    --background: #262626;
  }
}
</style>
