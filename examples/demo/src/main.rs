use bevy::input::common_conditions::input_pressed;
use bevy::prelude::*;
use bevy::window::PrimaryWindow;
use bevy_cef::prelude::*;

fn main() {
    App::new()
        .add_plugins((DefaultPlugins, CefPlugin))
        .add_systems(
            Startup,
            (
                spawn_camera,
                spawn_directional_light,
                spawn_github_webview,
                spawn_github_io_webview,
                spawn_ground,
                enable_ime,
            ),
        )
        .insert_resource(AmbientLight::default())
        .add_systems(
            Update,
            (
                walk::<1, 0>.run_if(input_pressed(KeyCode::ArrowRight)),
                walk::<-1, 0>.run_if(input_pressed(KeyCode::ArrowLeft)),
                walk::<0, 1>.run_if(input_pressed(KeyCode::ArrowUp)),
                walk::<0, -1>.run_if(input_pressed(KeyCode::ArrowDown)),
                rotate_camera::<1>.run_if(input_pressed(KeyCode::Digit1)),
                rotate_camera::<-1>.run_if(input_pressed(KeyCode::Digit2)),
            ),
        )
        .run();
}

fn spawn_camera(mut commands: Commands) {
    commands.spawn(Camera3d::default());
}

fn spawn_directional_light(mut commands: Commands) {
    commands.spawn((
        DirectionalLight {
            shadows_enabled: true,
            ..default()
        },
        Transform::from_translation(Vec3::new(1., 1., 1.)).looking_at(Vec3::ZERO, Vec3::Y),
    ));
}

fn enable_ime(mut primary_window: Query<&mut Window, With<PrimaryWindow>>) {
    primary_window.single_mut().unwrap().ime_enabled = true;
}

fn spawn_github_webview(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<WebviewExtendStandardMaterial>>,
) {
    commands.spawn((
        CefWebviewUri::new("https://github.com/not-elm/bevy_cef"),
        Mesh3d(meshes.add(Plane3d::new(Vec3::Z, Vec2::ONE))),
        WebviewSize(Vec2::splat(800.0)),
        MeshMaterial3d(materials.add(WebviewExtendStandardMaterial::default())),
        Transform::from_translation(Vec3::new(1.5, 0., -4.0)),
    ));
}

fn spawn_github_io_webview(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<WebviewExtendStandardMaterial>>,
) {
    commands.spawn((
        CefWebviewUri::new("https://not-elm.github.io/bevy_cef"),
        WebviewSize(Vec2::splat(800.0)),
        Mesh3d(meshes.add(Plane3d::new(Vec3::Z, Vec2::ONE))),
        MeshMaterial3d(materials.add(WebviewExtendStandardMaterial::default())),
        Transform::from_translation(Vec3::new(-1.5, 0., -4.0)), // .with_rotation(Quat::from_rotation_y(-90f32.to_radians())),
    ));
}

fn spawn_ground(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    commands.spawn((
        Mesh3d(meshes.add(Plane3d::new(Vec3::Y, Vec2::new(10., 10.)))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: Color::srgba(0.8, 0.8, 0.8, 1.0),
            ..default()
        })),
        Transform::from_translation(Vec3::new(0., -2., 0.)),
    ));
}

fn walk<const X: isize, const Z: isize>(
    mut q: Query<&mut Transform, With<Camera3d>>,
    time: Res<Time>,
) {
    for mut t in &mut q {
        const SPEED: f32 = 1.5; // 調整可
        let dt = time.delta_secs();

        let up = Vec3::Y;
        let mut f = t.forward().as_vec3();
        f = (f - up * f.dot(up)).normalize_or_zero();
        let r = f.cross(up).normalize_or_zero();

        let input = Vec2::new(X as f32, Z as f32);
        if input.length_squared() > 0.0 {
            let dir = (r * input.x + f * input.y).normalize_or_zero();
            t.translation += dir * SPEED * dt;
        }
    }
}

fn rotate_camera<const X: isize>(
    mut transforms: Query<&mut Transform, With<Camera3d>>,
    time: Res<Time>,
) {
    for mut transform in transforms.iter_mut() {
        const SPEED: f32 = 1.0;
        let rotation = Quat::from_rotation_y(SPEED * time.delta_secs() * X as f32);
        transform.rotation *= rotation;
    }
}
