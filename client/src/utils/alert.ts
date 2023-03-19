import {
  alertController,
  createAnimation,
  type AlertButton,
  type Animation
} from '@ionic/vue';

export interface AlertOptions {
  title: string;
  message: string;
  buttons: keyof typeof BUTTONS;
}

function enterAnimation(baseEl: any): Animation {
  const root = baseEl;

  const backdropAnimation = createAnimation()
    .addElement(root.querySelector('ion-backdrop')!)
    .fromTo('opacity', '0.01', 'var(--backdrop-opacity)');

  const wrapperAnimation = createAnimation()
    .addElement(root.querySelector('.alert-wrapper')!)
    .keyframes([
      { offset: 0, opacity: '0', transform: 'scale(0.5)' },
      { offset: 1, opacity: '0.99', transform: 'scale(1)' }
    ]);

  return createAnimation()
    .addElement(baseEl)
    .easing('cubic-bezier(0.4, 0, 0.2, 1)')
    .duration(200)
    .addAnimation([backdropAnimation, wrapperAnimation]);
}

function leaveAnimation(baseEl: any): Animation {
  return enterAnimation(baseEl).direction('reverse');
}

const BUTTONS = {
  'delete-cancel': [
    {
      text: 'Cancel',
      role: 'cancel'
    },
    {
      text: 'Delete',
      role: 'destructive'
    }
  ],
  'keep-discard': [
    {
      text: 'Discard',
      role: 'cancel'
    },
    {
      text: 'Keep',
      role: 'keep'
    }
  ],
  'ok': [
    {
      text: 'OK',
      role: 'ok'
    }
  ]
} satisfies { [key: string]: AlertButton[] };

export async function showAlert(options: AlertOptions): Promise<boolean> {
  const alert = await alertController.create({
    header: options.title,
    message: options.message,
    buttons: BUTTONS[options.buttons],
    enterAnimation,
    leaveAnimation,
  });

  await alert.present();
  const role = (await alert.onDidDismiss()).role;

  return !!role && role !== 'cancel' && role !== 'backdrop';
}
